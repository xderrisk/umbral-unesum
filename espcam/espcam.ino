#define ENABLE_DATABASE
#define ENABLE_USER_AUTH
#define ENABLE_ACCESS_TOKEN

#include "esp_camera.h"
#include "secret.h"
#include <ArduinoJson.h>
#include <FirebaseClient.h>
#include <PubSubClient.h>
#include <WiFi.h>
#include <WiFiClientSecure.h>

// ========== CONFIGURACIÓN OPTIMIZADA ==========
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

// Parámetros optimizados para detección en aulas
#define DIFF_THRESHOLD 30        // Aumentado para reducir falsos positivos
#define CHANGE_PERCENT 0.12f     // Ajustado para mejor sensibilidad en aulas
#define MIN_CHANGE_PIXELS 200    // Umbral mínimo de píxeles cambiados
#define MQTT_INTERVAL 5000       // Mayor intervalo para reducir tráfico
#define HEARTBEAT_INTERVAL 60000 // 1 minuto
#define MOTION_COOLDOWN 3000     // Cooldown después de detectar movimiento
#define MIN_FRAME_INTERVAL 200   // Mínimo entre capturas (5 fps)

// ========== VARIABLES GLOBALES ==========
String macAddress = "";
String previousState = "0";
String currentState = "0";
unsigned long lastAnalysis = 0;
unsigned long lastHeartbeat = 0;
unsigned long lastMQTTAttempt = 0;
unsigned long lastMotionTime = 0;
unsigned long lastFrameCapture = 0;
uint8_t motionCounter = 0; // Para filtro de persistencia
bool stateChanged = false;

static uint8_t *prevFrame = NULL;
static size_t prevLen = 0;

// ========== WiFi ==========
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
unsigned long lastFirebaseReconnect = 0;

// ========== FUNCIONES DE CONEXIÓN ==========
void reconnectMQTT() {
  unsigned long now = millis();
  if (now - lastMQTTAttempt < 15000)
    return;
  lastMQTTAttempt = now;

  if (!mqttClient.connected()) {
    Serial.print("MQTT connecting...");
    if (mqttClient.connect(macAddress.c_str())) {
      Serial.println("OK");
    } else {
      Serial.print("Failed (");
      Serial.print(mqttClient.state());
      Serial.println(")");
    }
  }
}

// ========== FIREBASE ==========
void processData(AsyncResult &result) {
  if (!result.isResult())
    return;
  if (result.isError()) {
    Serial.print("Firebase error: ");
    Serial.println(result.error().message().c_str());
    firebaseReady = false;
  } else {
    firebaseReady = true;
  }
}

void initFirebase() {
  String email = "camera_" + macAddress + "@umbral.unesum.edu";
  String password = "Umbral." + macAddress + "#";

  Serial.print("Firebase auth...");
  ssl_client.setInsecure();
  UserAuth userAuth(FIREBASE_API_KEY, email.c_str(), password.c_str());
  initializeApp(async_client, app, getAuth(userAuth), processData, "authTask");

  int attempts = 0;
  while (!app.ready() && attempts < 30) {
    delay(500);
    Serial.print(".");
    attempts++;
  }

  if (!app.ready()) {
    Serial.println(" FAILED");
    firebaseReady = false;
    return;
  }

  Serial.println(" OK");
  firebaseUid = String(app.getUid());
  app.getApp<RealtimeDatabase>(db);
  db.url(FIREBASE_URL);

  String path = "/cameras/" + firebaseUid;
  db.set<int>(async_client, path + "/status", 1, processData);

  writer.create(timestampJson, ".sv", string_t("timestamp"));
  db.set<object_t>(async_client, path + "/last_connection", timestampJson,
                   processData);

  Serial.println("Firebase ready");
  firebaseReady = true;
  lastFirebaseReconnect = millis();
}

void updateFirebaseState(String state) {
  if (!firebaseReady || firebaseUid.length() == 0)
    return;
  String path = "/cameras/" + firebaseUid;
  db.set<String>(async_client, path + "/status", state, processData);
  Serial.println("Firebase state updated: " + state);
}

void updateFirebaseHeartbeat() {
  if (!firebaseReady || firebaseUid.length() == 0)
    return;

  unsigned long now = millis();
  if (now - lastHeartbeat > HEARTBEAT_INTERVAL) {
    lastHeartbeat = now;
    String path = "/cameras/" + firebaseUid;
    object_t heartbeat;
    writer.create(heartbeat, ".sv", string_t("timestamp"));
    db.set<object_t>(async_client, path + "/last_connection", heartbeat,
                     processData);
    Serial.println("Firebase heartbeat sent");
  }
}

void processFirebase() {
  if (app.ready()) {
    app.loop();
    db.loop();
    updateFirebaseHeartbeat();
  } else {
    unsigned long now = millis();
    if (now - lastFirebaseReconnect > 60000) {
      lastFirebaseReconnect = now;
      Serial.println("Reconnecting Firebase...");
      initFirebase();
    }
  }
}

// ========== CÁMARA OPTIMIZADA ==========
void initCamera() {
  camera_config_t config = {
      .pin_pwdn = PWDN_GPIO_NUM,
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
      .frame_size = FRAMESIZE_QQVGA, // 160x120 - balance calidad/velocidad
      .jpeg_quality = 10,
      .fb_count = psramFound() ? 2 : 1,
      .grab_mode = CAMERA_GRAB_WHEN_EMPTY};

  esp_err_t err = esp_camera_init(&config);
  if (err != ESP_OK) {
    Serial.printf("Camera init failed: 0x%x\n", err);
    return;
  }
  Serial.println("Camera ready");
}

// ========== DETECCIÓN DE MOVIMIENTO OPTIMIZADA ==========
String detectMotion() {
  unsigned long now = millis();

  // Control de frecuencia de captura
  if (now - lastFrameCapture < MIN_FRAME_INTERVAL) {
    return currentState; // Retorna el último estado conocido
  }
  lastFrameCapture = now;

  camera_fb_t *fb = esp_camera_fb_get();
  if (!fb)
    return "ERR";

  // Inicializar frame de referencia
  if (!prevFrame) {
    prevLen = fb->len;
    prevFrame = (uint8_t *)malloc(prevLen);
    if (prevFrame) {
      memcpy(prevFrame, fb->buf, prevLen);
    }
    esp_camera_fb_return(fb);
    return "0";
  }

  // Detección optimizada con submuestreo
  uint32_t changedPixels = 0;
  uint32_t totalPixels = fb->len;

  // Submuestreo: analizar 1 de cada 4 píxeles para mayor velocidad
  for (size_t i = 0; i < totalPixels; i += 4) {
    int diff = fb->buf[i] - prevFrame[i];
    if (diff < 0)
      diff = -diff;
    if (diff > DIFF_THRESHOLD) {
      changedPixels++;
    }
  }

  // Actualizar frame de referencia
  memcpy(prevFrame, fb->buf, prevLen);
  esp_camera_fb_return(fb);

  // Escalar el conteo de píxeles cambiados (por el submuestreo)
  changedPixels *= 4;

  // Verificar umbral mínimo de píxeles
  if (changedPixels < MIN_CHANGE_PIXELS) {
    return "0";
  }

  float changeRatio = (float)changedPixels / totalPixels;

  // Filtro de persistencia: requiere múltiples detecciones consecutivas
  if (changeRatio > CHANGE_PERCENT) {
    motionCounter++;
    if (motionCounter >=
        3) { // 3 detecciones consecutivas = movimiento confirmado
      motionCounter = 0;
      lastMotionTime = now;
      return "1";
    }
  } else {
    motionCounter = 0;
  }

  return "0";
}

// ========== PUBLICACIÓN OPTIMIZADA ==========
void publishMQTT() {
  unsigned long now = millis();

  // Cooldown después de detectar movimiento
  if (now - lastMotionTime < MOTION_COOLDOWN) {
    if (currentState == "1") {
      // Mantener estado de movimiento durante el cooldown
      return;
    }
  }

  if (now - lastAnalysis < MQTT_INTERVAL)
    return;
  lastAnalysis = now;

  String newState = detectMotion();
  if (newState == "ERR")
    return;

  // Solo publicar si hay cambio de estado
  if (newState != currentState) {
    currentState = newState;
    stateChanged = true;

    JsonDocument doc;
    doc["mac"] = macAddress;
    doc["status"] = currentState;
    String payload;
    serializeJson(doc, payload);

    Serial.print("State changed: ");
    Serial.println(payload);

    // Intentar MQTT
    if (mqttClient.connected()) {
      if (mqttClient.publish("unesum/classrooms", payload.c_str())) {
        Serial.println("MQTT sent");
      } else {
        Serial.println("MQTT send failed");
      }
    }

    // Actualizar Firebase siempre
    updateFirebaseState(currentState);
  }
}

// ========== SETUP ==========
void setup() {
  Serial.begin(115200);
  delay(1000);

  // WiFi
  WiFi.begin(SECRET_SSID, SECRET_PASS);
  Serial.print("Connecting WiFi");
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }

  macAddress = WiFi.macAddress();
  macAddress.replace(":", "");
  macAddress.toLowerCase();

  Serial.println("\nWiFi ready");
  Serial.print("IP: ");
  Serial.println(WiFi.localIP());
  Serial.print("MAC: ");
  Serial.println(macAddress);

  // MQTT
  mqttClient.setServer(SECRET_MQTT_BROKER, 1883);

  // Firebase
  initFirebase();

  // Cámara
  initCamera();

  // Estado inicial
  currentState = "0";
  previousState = "0";

  Serial.println("System ready - Optimized for classroom use");
}

// ========== LOOP OPTIMIZADO ==========
void loop() {
  // MQTT - no bloqueante
  reconnectMQTT();
  if (mqttClient.connected()) {
    mqttClient.loop();
  }

  // Publicar estado (optimizado)
  publishMQTT();

  // Firebase - siempre activo
  processFirebase();

  // Pequeña pausa para no saturar el CPU
  delay(20);
}
