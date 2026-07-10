#define ENABLE_DATABASE
#define ENABLE_USER_AUTH
#define ENABLE_ACCESS_TOKEN

#include "esp_camera.h"
#include "secret.h"
#include <ArduinoJson.h>
#include <ESPmDNS.h>
#include <FirebaseClient.h>
#include <PubSubClient.h>
#include <WiFi.h>
#include <WiFiClientSecure.h>

// ========== CONFIGURACIÓN DE LA CÁMARA (ESP32-CAM AI-THINKER) ==========
#define PWDN_GPIO_NUM 32
#define RESET_GPIO_NUM -1
#define XCLK_GPIO_NUM 0
#define SIOD_GPIO_NUM 26
#define SIOC_GPIO_NUM 27
#define Y9_GPIO_NUM 35
#define Y8_GPIO_NUM 34
#define Y7_GPIO_NUM 39
#define Y6_GPIO_NUM 36
#define Y5_GPIO_NUM 21
#define Y4_GPIO_NUM 19
#define Y3_GPIO_NUM 18
#define Y2_GPIO_NUM 5
#define VSYNC_GPIO_NUM 25
#define HREF_GPIO_NUM 23
#define PCLK_GPIO_NUM 22

#define DIFF_THRESHOLD 20
#define CHANGE_PERCENT 0.12f
#define MQTT_INTERVAL 1000
#define HEARTBEAT_INTERVAL 30000

// ========== CONFIGURACIÓN DE OCUPACIÓN ==========
#define OCCUPIED_TIMEOUT 60000 // 30 segundos para marcar como desocupada
#define MOTION_COOLDOWN 2000   // Cooldown entre detecciones de movimiento

// ========== VARIABLES GLOBALES ==========
String macAddress = "";
String previousState = "0"; // 0 = desocupada, 1 = ocupada
unsigned long lastAnalysis = 0;
unsigned long lastHeartbeat = 0;
unsigned long lastMQTTAttempt = 0;
unsigned long lastMDNSDiscovery = 0;
unsigned long lastMotionTime = 0; // Último momento en que se detectó movimiento
unsigned long lastOccupiedUpdate = 0; // Última actualización de estado ocupado

// Gestión limpia de memoria de imagen
static uint8_t *prevFrame = NULL;
static size_t prevLen = 0;

// ========== WiFi & MQTT ==========
WiFiClient espClient;
PubSubClient mqttClient(espClient);

// ========== Firebase ==========
WiFiClientSecure ssl_client;
AsyncClientClass async_client(ssl_client);
FirebaseApp app;
RealtimeDatabase db;
JsonWriter writer;
object_t timestampJson;
String firebaseUid = "";
bool firebaseReady = false;

// Variables de control de estado asíncrono para Firebase
bool firebaseConnecting = false;
unsigned long firebaseAuthStart = 0;

// ========== COMPORTAMIENTO MQTT (SIN BLOQUEOS) ==========
void discoverMQTT() {
  Serial.println("Buscando broker MQTT via mDNS...");
  int svc = MDNS.queryService("umbral-mqtt", "tcp");
  if (svc > 0) {
    mqttClient.setServer(MDNS.address(0), MDNS.port(0));
    Serial.printf("Broker MQTT descubierto: %s:%d\n",
                  MDNS.address(0).toString().c_str(), MDNS.port(0));
  } else {
    Serial.println("mDNS: Servidor MQTT no encontrado en la red local.");
  }
}

void reconnectMQTT() {
  if (mqttClient.connected())
    return;

  unsigned long now = millis();
  if (now - lastMQTTAttempt < 15000)
    return;
  lastMQTTAttempt = now;

  if (firebaseConnecting)
    return;

  if (now - lastMDNSDiscovery > 30000) {
    discoverMQTT();
    lastMDNSDiscovery = now;
  }

  int svc = MDNS.queryService("umbral-mqtt", "tcp");
  if (svc > 0) {
    mqttClient.setServer(MDNS.address(0), MDNS.port(0));
    Serial.printf("mDNS: Broker MQTT detectado dinámicamente en: %s:%d\n",
                  MDNS.address(0).toString().c_str(), MDNS.port(0));
  } else {
    Serial.println("mDNS: El broker sigue sin aparecer en la red.");
    return;
  }

  Serial.print("Intentando conexión MQTT...");
  if (mqttClient.connect(macAddress.c_str())) {
    Serial.println("CONECTADO");
  } else {
    Serial.printf("FALLÓ (rc=%d). Reintento en 15s.\n", mqttClient.state());
  }
}

// ========== COMPORTAMIENTO FIREBASE (100% ASÍNCRONO) ==========
void processData(AsyncResult &result) {
  if (!result.isResult())
    return;
  if (result.isError()) {
    Serial.printf("Error de Firebase: %s\n", result.error().message().c_str());
    firebaseReady = false;
  } else {
    firebaseReady = true;
  }
}

void initFirebaseAsync() {
  if (firebaseConnecting)
    return;

  String email = "camera_" + macAddress + "@umbral.unesum.edu";
  String password = "Umbral." + macAddress + "#";

  Serial.println("Iniciando Autenticación de Firebase (Asíncrona)...");

  ssl_client.setInsecure();

  UserAuth userAuth(FIREBASE_API_KEY, email.c_str(), password.c_str());

  initializeApp(async_client, app, getAuth(userAuth), processData, "authTask");

  firebaseConnecting = true;
  firebaseAuthStart = millis();
}

void updateFirebaseState(String state) {
  if (!firebaseReady || firebaseUid.length() == 0)
    return;
  String path = "/cameras/" + firebaseUid;
  db.set<String>(async_client, path + "/status", state, processData);
  Serial.println("Firebase: Estado actualizado -> " + state);
}

void updateFirebaseHeartbeat() {
  if (!firebaseReady || firebaseUid.length() == 0)
    return;

  if (millis() - lastHeartbeat > HEARTBEAT_INTERVAL) {
    lastHeartbeat = millis();
    String path = "/cameras/" + firebaseUid;
    object_t heartbeat;
    writer.create(heartbeat, ".sv", string_t("timestamp"));
    db.set<object_t>(async_client, path + "/last_connection", heartbeat,
                     processData);
    Serial.println("Firebase: Heartbeat enviado.");
  }
}

void processFirebase() {
  if (app.ready()) {
    if (firebaseConnecting) {
      Serial.println("\n¡Autenticación de Firebase EXITOSA!");
      firebaseConnecting = false;
      firebaseReady = true;

      firebaseUid = String(app.getUid());
      app.getApp<RealtimeDatabase>(db);
      db.url(FIREBASE_URL);

      String path = "/cameras/" + firebaseUid;
      db.set<String>(async_client, path + "/status", "0", processData);

      writer.create(timestampJson, ".sv", string_t("timestamp"));
      db.set<object_t>(async_client, path + "/last_connection", timestampJson,
                       processData);
    }

    app.loop();
    db.loop();
    updateFirebaseHeartbeat();
  } else if (firebaseConnecting) {
    if (millis() - firebaseAuthStart > 20000) {
      Serial.println("\nFirebase Auth TIMEOUT. Servidor posiblemente caído.");
      firebaseConnecting = false;
      firebaseReady = false;
    } else {
      static unsigned long lastDot = 0;
      if (millis() - lastDot > 1000) {
        lastDot = millis();
        Serial.print(".");
      }
    }
  } else {
    firebaseReady = false;
    static unsigned long lastFirebaseReconnectAttempt = 0;
    if (millis() - lastFirebaseReconnectAttempt > 30000) {
      lastFirebaseReconnectAttempt = millis();
      Serial.println(
          "Firebase fuera de línea. Reintentando conectar de fondo...");
      initFirebaseAsync();
    }
  }
}

// ========== PROCESAMIENTO DE LA CÁMARA (SEGURO) ==========
void initCamera() {
  camera_config_t config = {.pin_pwdn = PWDN_GPIO_NUM,
                            .pin_reset = RESET_GPIO_NUM,
                            .pin_xclk = XCLK_GPIO_NUM,
                            .pin_sscb_sda = SIOD_GPIO_NUM,
                            .pin_sscb_scl = SIOC_GPIO_NUM,
                            .pin_d7 = Y9_GPIO_NUM,
                            .pin_d6 = Y8_GPIO_NUM,
                            .pin_d5 = Y7_GPIO_NUM,
                            .pin_d4 = Y6_GPIO_NUM,
                            .pin_d3 = Y5_GPIO_NUM,
                            .pin_d2 = Y4_GPIO_NUM,
                            .pin_d1 = Y3_GPIO_NUM,
                            .pin_d0 = Y2_GPIO_NUM,
                            .pin_vsync = VSYNC_GPIO_NUM,
                            .pin_href = HREF_GPIO_NUM,
                            .pin_pclk = PCLK_GPIO_NUM,
                            .xclk_freq_hz = 20000000,
                            .ledc_timer = LEDC_TIMER_0,
                            .ledc_channel = LEDC_CHANNEL_0,
                            .pixel_format = PIXFORMAT_GRAYSCALE,
                            .frame_size = FRAMESIZE_QQVGA,
                            .jpeg_quality = 12,
                            .fb_count = psramFound() ? 2 : 1,
                            .grab_mode = CAMERA_GRAB_WHEN_EMPTY};

  esp_err_t err = esp_camera_init(&config);
  if (err != ESP_OK) {
    Serial.printf("Fallo al inicializar la cámara: 0x%x\n", err);
    return;
  }
  Serial.println("Cámara lista y configurada en Grayscale QQVGA");
}

String detectMotion() {
  camera_fb_t *fb = esp_camera_fb_get();
  if (!fb)
    return "ERR";

  if (!prevFrame || prevLen != fb->len) {
    if (prevFrame)
      free(prevFrame);
    prevLen = fb->len;
    prevFrame = (uint8_t *)malloc(prevLen);
    if (prevFrame) {
      memcpy(prevFrame, fb->buf, prevLen);
    }
    esp_camera_fb_return(fb);
    return "0";
  }

  uint32_t changedPixels = 0;
  for (size_t i = 0; i < fb->len; i++) {
    if (abs(fb->buf[i] - prevFrame[i]) > DIFF_THRESHOLD) {
      changedPixels++;
    }
  }

  memcpy(prevFrame, fb->buf, prevLen);
  esp_camera_fb_return(fb);

  float changeRatio = (float)changedPixels / fb->len;
  return (changeRatio > CHANGE_PERCENT) ? "1" : "0";
}

// ========== NUEVA LÓGICA DE OCUPACIÓN ==========
void updateOccupancy(String motionDetected) {
  unsigned long now = millis();

  // Si se detecta movimiento, actualizar el tiempo de última detección
  if (motionDetected == "1") {
    lastMotionTime = now;

    // Si estaba desocupada, marcar como ocupada inmediatamente
    if (previousState == "0") {
      previousState = "1";
      lastOccupiedUpdate = now;
      publishState("1");
      Serial.println("✅ Aula OCUPADA - Movimiento detectado");
    } else {
      // Ya estaba ocupada, actualizar timestamp
      lastOccupiedUpdate = now;
    }
    return;
  }

  // Si no hay movimiento y está ocupada, verificar timeout
  if (previousState == "1" && motionDetected == "0") {
    // Verificar si ha pasado el tiempo de timeout (30 segundos)
    if (now - lastOccupiedUpdate >= OCCUPIED_TIMEOUT) {
      previousState = "0";
      publishState("0");
      Serial.println("❌ Aula DESOCUPADA - 30 segundos sin movimiento");
    }
  }
}

// ========== PUBLICACIÓN DE RESULTADOS ==========
void publishState(String state) {
  // Publicar por MQTT
  StaticJsonDocument<128> doc;
  doc["mac"] = macAddress;
  doc["status"] = state;
  String payload;
  serializeJson(doc, payload);

  if (mqttClient.connected()) {
    if (mqttClient.publish("unesum/classrooms", payload.c_str())) {
      Serial.println("MQTT: Estado enviado con éxito.");
    } else {
      Serial.println("MQTT: Error al enviar.");
    }
  } else {
    Serial.println("MQTT: Desconectado. Mensaje omitido localmente.");
  }

  // Publicar en Firebase
  updateFirebaseState(state);
}

void publishMQTT() {
  unsigned long now = millis();

  // Control de frecuencia de análisis (cada 3 segundos)
  if (now - lastAnalysis < MQTT_INTERVAL)
    return;
  lastAnalysis = now;

  // Detectar movimiento
  String motionDetected = detectMotion();
  if (motionDetected == "ERR")
    return;

  // Actualizar lógica de ocupación
  updateOccupancy(motionDetected);
}

// ========== CONFIGURACIÓN INICIAL (SETUP) ==========
void setup() {
  Serial.begin(115200);
  delay(1000);

  WiFi.begin(SECRET_SSID, SECRET_PASS);
  Serial.print("Conectando a la red Wi-Fi");

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }

  macAddress = WiFi.macAddress();
  macAddress.replace(":", "");
  macAddress.toLowerCase();

  Serial.println("\nWi-Fi Conectado.");
  Serial.printf("IP Local: %s | MAC ID Único: %s\n",
                WiFi.localIP().toString().c_str(), macAddress.c_str());

  // Configuración inicial de mDNS rápida
  MDNS.begin("umbral-cam");
  lastMDNSDiscovery = millis();
  int svc = 0;
  for (int i = 0; i < 3 && svc == 0; i++) {
    svc = MDNS.queryService("umbral-mqtt", "tcp");
    if (svc == 0 && i < 2)
      delay(1000);
  }

  if (svc > 0) {
    mqttClient.setServer(MDNS.address(0), MDNS.port(0));
    Serial.printf("Broker MQTT inicial detectado en: %s:%d\n",
                  MDNS.address(0).toString().c_str(), MDNS.port(0));
  } else {
    Serial.println("No se encontró broker MQTT en el arranque. Se buscará en "
                   "segundo plano.");
  }

  // Inicializar Firebase de forma asíncrona
  initFirebaseAsync();

  // Inicializar hardware de la cámara
  initCamera();

  // Estado inicial: desocupada
  previousState = "0";
  lastMotionTime = millis();
  lastOccupiedUpdate = millis();

  Serial.println("Sistema en marcha - Detección de ocupación activa");
  Serial.println("Tiempo para marcar como desocupada: 30 segundos");
}

// ========== BUCLE PRINCIPAL (LOOP) ==========
void loop() {
  // Gestión de MQTT no bloqueante
  reconnectMQTT();
  if (mqttClient.connected()) {
    mqttClient.loop();
  }

  // Captura y lógica de detección de ocupación
  publishMQTT();

  // Ejecución de tareas internas asíncronas de Firebase
  processFirebase();

  delay(30); // 30ms permite un descanso al procesador
}
