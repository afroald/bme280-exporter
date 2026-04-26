variable "REGISTRY" { default = "" }

target "default" {
  dockerfile = "Dockerfile"
  platforms  = ["linux/arm/v7"]
  tags       = ["${REGISTRY}/bme280-exporter"]
}
