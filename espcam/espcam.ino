#define ENABLE_DATABASE
#define ENABLE_USER_AUTH
#define ENABLE_ACCESS_TOKEN

#include "esp_camera.h"
#include "secret.h"
#include <ArduinoJson.h>
#include <FirebaseClient.h>
#include <ESPmDNS.h>
#include <PubSubClient.h>
#include <WiFi.h>
#include <WiFiClientSecure.h>

// ========== CAMERA CONFIG ==========
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

#define DIFF_THRESHOLD 25
#define CHANGE_PERCENT 0.15f
#define MQTT_INTERVAL 3000
#define HEARTBEAT_INTERVAL 30000

// ========== GLOBAL VARIABLES ==========
String macAddress = "";
String previousState = "";
unsigned long lastAnalysis = 0;
unsigned long lastHeartbeat = 0;
unsigned long lastMQTTAttempt = 0;

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

// ========== MQTT ==========
void reconnectMQTT() {
  unsigned long now = millis();
  // Intentar reconectar cada 10 segundos máximo
  if (now - lastMQTTAttempt < 10000)
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

  if (millis() - lastHeartbeat > HEARTBEAT_INTERVAL) {
    lastHeartbeat = millis();
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
    // Intentar reconectar Firebase si se perdió
    static unsigned long lastReconnect = 0;
    if (millis() - lastReconnect > 60000) {
      lastReconnect = millis();
      Serial.println("Reconnecting Firebase...");
      initFirebase();
    }
  }
}

// ========== CAMERA ==========
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
    Serial.printf("Camera init failed: 0x%x\n", err);
    return;
  }
  Serial.println("Camera ready");
}

String detectMotion() {
  camera_fb_t *fb = esp_camera_fb_get();
  if (!fb)
    return "ERR";

  if (!prevFrame) {
    prevLen = fb->len;
    prevFrame = (uint8_t *)malloc(prevLen);
    if (prevFrame)
      memcpy(prevFrame, fb->buf, prevLen);
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

// ========== MQTT PUBLISH ==========
void publishMQTT() {
  unsigned long now = millis();
  if (now - lastAnalysis < MQTT_INTERVAL)
    return;

  lastAnalysis = now;
  String currentState = detectMotion();

  if (currentState == "ERR")
    return;

  if (currentState != previousState) {
    previousState = currentState;

    JsonDocument doc;
    doc["mac"] = macAddress;
    doc["status"] = currentState;
    String payload;
    serializeJson(doc, payload);

    Serial.print("State changed: ");
    Serial.println(payload);

    // Intentar MQTT pero no bloquear
    if (mqttClient.connected()) {
      if (mqttClient.publish("unesum/classrooms", payload.c_str())) {
        Serial.println("MQTT sent");
      } else {
        Serial.println("MQTT send failed");
      }
    } else {
      Serial.println("MQTT not connected");
    }

    // SIEMPRE actualizar Firebase independientemente de MQTT
    updateFirebaseState(currentState);
  }
}

// ========== SETUP ==========
void setup() {
  Serial.begin(115200);
  delay(1000);

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

  // Inicializar MQTT con descubrimiento mDNS
  MDNS.begin("umbral-cam");
  int svc = 0;
  for (int i = 0; i < 3 && svc == 0; i++) {
    svc = MDNS.queryService("umbral-mqtt", "tcp");
    if (svc == 0 && i < 2) delay(2000);
  }
  if (svc > 0) {
    mqttClient.setServer(MDNS.address(0), MDNS.port(0));
    Serial.print("MQTT broker discovered via mDNS: ");
    Serial.print(MDNS.address(0));
    Serial.print(":");
    Serial.println(MDNS.port(0));
  } else {
    Serial.println("No MQTT broker found via mDNS");
  }

  // Inicializar Firebase (esto es lo importante)
  initFirebase();

  // Inicializar cámara
  initCamera();

  Serial.println("System ready - Firebase active");
}

// ========== LOOP ==========
void loop() {
  // MQTT (no crítico)
  reconnectMQTT();
  if (mqttClient.connected()) {
    mqttClient.loop();
  }

  // Publicar estado (usa Firebase si MQTT falla)
  publishMQTT();

  // Firebase SIEMPRE corre independientemente
  processFirebase();

  delay(50);
}
