#include "secret.h"
#include <ArduinoJson.h>
#include <PubSubClient.h>
#include <WiFi.h>

// Wifi
const char *ssid = SECRET_SSID;
const char *password = SECRET_PASS;
WiFiClient espClient;
String macAddress = "";

// MQTT
PubSubClient client(espClient);
const char *mqtt_broker = SECRET_MQTT_BROKER;
const char *mqtt_topic = "unesum/aulas";

// Lógica de estado
bool estadoAula = false;
String estadoAnterior = "";
unsigned long ultimoAnalisis = 0;
const long intervalo = 3000;

void setup() {
  Serial.begin(115200);
  delay(1000);

  WiFi.begin(ssid, password);

  while (WiFi.status() != WL_CONNECTED) {
    delay(1000);
    Serial.print(".");
  }
  macAddress = WiFi.macAddress();
  Serial.println("Conexión exitosa");
  Serial.print("Dirección IP: ");
  Serial.println(WiFi.localIP());
  Serial.print("Dirección MAC: ");
  Serial.println(macAddress);
  client.setServer(mqtt_broker, 1883);
  initCamera();
}

void loop() {
  if (!client.connected()) {
    reconnect();
  }
  client.loop();
  sendMqtt();
}
