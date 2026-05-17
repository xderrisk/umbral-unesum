#include <WiFi.h>
#include <PubSubClient.h>
#include "secret.h"

const char* ssid = SECRET_SSID;
const char* password = SECRET_PASS;
const char* mqtt_broker = SECRET_MQTT_BROKER;
const char* mqtt_topic  = "unesum/aulas";
bool estadoAula = false;

WiFiClient espClient;
PubSubClient client(espClient);

String macAddress = "";
unsigned long ultimoMensaje = 0;

void reconnect() {
  while (!client.connected()) {
    Serial.print("Intentando conexión MQTT...");
    
    if (client.connect(macAddress.c_str())) {
      Serial.println("¡Conectado al Broker MQTT!");
    } else {
      Serial.print("Falló con estado: ");
      Serial.print(client.state());
      Serial.println(". Intentando de nuevo en 5 segundos...");
      delay(5000);
    }
  }
}

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
}

void loop() {
  if (!client.connected()) {
    reconnect();
  }
  client.loop();

  unsigned long ahora = millis();
  if (ahora - ultimoMensaje > 5000) {
    ultimoMensaje = ahora;
    estadoAula = !estadoAula;
    String estadoString = estadoAula ? "1" : "0";

    String payload = "{\"mac\":\"" + macAddress + "\", \"estado\":\"" + String(estadoString) + "\"}";
    
    Serial.print("Publicando mensaje: ");
    Serial.println(payload);

    // Publicar al tópico
    if (client.publish(mqtt_topic, payload.c_str())) {
      Serial.println("Mensaje enviado con éxito.");
    } else {
      Serial.println("Error al enviar el mensaje.");
    }
  }
}
