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

void sendMqtt() {
  unsigned long ahora = millis();
  if (ahora - ultimoAnalisis > intervalo) {
    ultimoAnalisis = ahora;
    String estadoActual = verificarEstadoAula();
    if (estadoActual == "ERR") {
      Serial.println(
          "Reintentando captura de frame en el próximo intervalo...");
      return;
    }
    if (estadoActual != estadoAnterior) {
      estadoAnterior = estadoActual;
      JsonDocument doc;
      doc["mac"] = macAddress;
      doc["estado"] = estadoActual;
      String payload;
      serializeJson(doc, payload);
      Serial.print("Cambio detectado. Publicando mensaje: ");
      Serial.println(payload);
      if (client.publish(mqtt_topic, payload.c_str())) {
        Serial.println("Mensaje enviado con éxito.");
      } else {
        Serial.println("Error crítico al enviar por MQTT.");
      }
    } else {
      Serial.println("El estado no ha cambiado (" + estadoActual + ").");
    }
  }
}
