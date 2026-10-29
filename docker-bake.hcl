group "default" {
  targets = ["giong"]
}

group "apps" {
  targets = ["cli", "controller", "reconciler", "console"]
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

target "reconciler-builder" {
  inherits = ["_common"]
  dockerfile = "apps/reconciler/Dockerfile"
  target = "builder"
}

target "reconciler" {
  inherits = ["reconciler-builder"]
  target = "runtime"
  tags = ["giong-reconciler:latest"]
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
    reconciler-builder = "target:reconciler-builder"
    console-builder = "target:console-builder"
  }
}
