group "default" {
  targets = ["giong", "plugin-repository", "plugin-flow", "plugin-kubectl"]
}

group "apps" {
  targets = ["cli", "controller", "console"]
}

group "plugins" {
  targets = ["plugin-repository", "plugin-flow", "plugin-kubectl"]
}

variable "GIONG_PLUGIN_IMAGE_PREFIX" {
  default = "giong-plugin-"
}

variable "GIONG_PLUGIN_IMAGE_TAG" {
  default = "latest"
}

target "docker-metadata-action" {
  tags = ["giong:latest"]
}

target "_common" {
  context = "."
}

target "cli-builder" {
  inherits = ["_common"]
  dockerfile = "apps/cli/Dockerfile"
  target = "builder"
}

target "cli" {
  inherits = ["cli-builder"]
  target = "runtime"
  tags = ["giong-cli:latest"]
}

target "controller-builder" {
  inherits = ["_common"]
  dockerfile = "apps/controller/Dockerfile"
  target = "builder"
}

target "controller" {
  inherits = ["controller-builder"]
  target = "runtime"
  tags = ["giong-controller:latest"]
}

target "console-builder" {
  inherits = ["_common"]
  dockerfile = "apps/console/Dockerfile"
  target = "builder"
}

target "console" {
  inherits = ["console-builder"]
  target = "runtime"
  tags = ["giong-console:latest"]
}

target "giong" {
  inherits = ["_common", "docker-metadata-action"]
  dockerfile = "Dockerfile"
  contexts = {
    cli-builder = "target:cli-builder"
    controller-builder = "target:controller-builder"
    console-builder = "target:console-builder"
  }
}

target "plugin-repository" {
  inherits = ["_common"]
  dockerfile = "plugins/repository/Dockerfile"
  tags = ["${GIONG_PLUGIN_IMAGE_PREFIX}repository:${GIONG_PLUGIN_IMAGE_TAG}"]
}

target "plugin-flow" {
  inherits = ["_common"]
  dockerfile = "plugins/flow/Dockerfile"
  tags = ["${GIONG_PLUGIN_IMAGE_PREFIX}flow:${GIONG_PLUGIN_IMAGE_TAG}"]
}

target "plugin-kubectl" {
  inherits = ["_common"]
  dockerfile = "plugins/kubectl/Dockerfile"
  tags = ["${GIONG_PLUGIN_IMAGE_PREFIX}kubectl:${GIONG_PLUGIN_IMAGE_TAG}"]
}
