# audio-recorder

A Helm chart for deploying [audio-recorder](https://github.com/dhpollack/audio-recorder) — a Rust/Leptos audio recorder with on-device Whisper transcription running on Cloudflare Workers.

## Install

```bash
helm install audio-recorder oci://ghcr.io/dhpollack/audio-recorder --version 0.0.1
```

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `replicaCount` | int | `1` | Number of replicas |
| `image.repository` | string | `ghcr.io/dpollack/audio-recorder` | Image repository |
| `image.tag` | string | `latest` | Image tag |
| `image.pullPolicy` | string | `IfNotPresent` | Image pull policy |
| `nameOverride` | string | `""` | Override the name used in resource names |
| `fullnameOverride` | string | `""` | Override the full name used in resource names |
| `serviceAccount.create` | bool | `true` | Create a service account |
| `service.port` | int | `8080` | Service port |
| `gateway.httproute.create` | bool | `false` | Create a Gateway API HTTPRoute |
| `gateway.httproute.hostname` | string | `audio-recorder.example.com` | Hostname for the HTTPRoute |
| `gateway.httproute.parentRef.name` | string | `external-http` | Gateway parent ref name |
| `gateway.httproute.parentRef.namespace` | string | `gateway` | Gateway parent ref namespace |
| `gateway.httproute.parentRef.sectionName` | string | `""` | Gateway parent ref section name (omitted if empty) |
