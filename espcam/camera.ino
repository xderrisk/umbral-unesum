#include "esp_camera.h"

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

uint8_t *prev_frame = NULL;
size_t prev_len = 0;

#define UMBRAL_DIFERENCIA 25
#define PORCENTAJE_CAMBIO 0.15

void initCamera() {
  camera_config_t config;
  config.ledc_channel = LEDC_CHANNEL_0;
  config.ledc_timer = LEDC_TIMER_0;
  config.pin_d0 = Y2_GPIO_NUM;
  config.pin_d1 = Y3_GPIO_NUM;
  config.pin_d2 = Y4_GPIO_NUM;
  config.pin_d3 = Y5_GPIO_NUM;
  config.pin_d4 = Y6_GPIO_NUM;
  config.pin_d5 = Y7_GPIO_NUM;
  config.pin_d6 = Y8_GPIO_NUM;
  config.pin_d7 = Y9_GPIO_NUM;
  config.pin_xclk = XCLK_GPIO_NUM;
  config.pin_pclk = PCLK_GPIO_NUM;
  config.pin_vsync = VSYNC_GPIO_NUM;
  config.pin_href = HREF_GPIO_NUM;
  config.pin_sscb_sda = SIOD_GPIO_NUM;
  config.pin_sscb_scl = SIOC_GPIO_NUM;
  config.pin_pwdn = PWDN_GPIO_NUM;
  config.pin_reset = RESET_GPIO_NUM;
  config.xclk_freq_hz = 20000000;
  config.pixel_format = PIXFORMAT_GRAYSCALE;
  config.frame_size = FRAMESIZE_QQVGA;

  if (psramFound()) {
    config.fb_count = 2;
  } else {
    config.fb_count = 1;
  }

  esp_err_t err = esp_camera_init(&config);
  if (err != ESP_OK) {
    Serial.printf("Error al iniciar la cámara: 0x%x\n", err);
    return;
  }
  Serial.println("Cámara en escala de grises lista para detección.");
}

String verificarEstadoAula() {
  camera_fb_t *fb = esp_camera_fb_get();
  if (!fb) {
    Serial.println("Error: No se pudo capturar frame");
    return "0";
  }

  if (prev_frame == NULL) {
    prev_len = fb->len;
    prev_frame = (uint8_t *)malloc(prev_len);
    if (prev_frame != NULL) {
      memcpy(prev_frame, fb->buf, prev_len);
    }
    esp_camera_fb_return(fb);
    return "0";
  }

  unsigned int pixeles_cambiados = 0;
  unsigned int total_pixeles = fb->len;

  for (size_t i = 0; i < total_pixeles; i++) {
    int diferencia = abs(fb->buf[i] - prev_frame[i]);
    if (diferencia > UMBRAL_DIFERENCIA) {
      pixeles_cambiados++;
    }
  }

  memcpy(prev_frame, fb->buf, prev_len);
  esp_camera_fb_return(fb);
  float ratio_cambio = (float)pixeles_cambiados / (float)total_pixeles;

  if (ratio_cambio > PORCENTAJE_CAMBIO) {
    return "1";
  } else {
    return "0";
  }
}
