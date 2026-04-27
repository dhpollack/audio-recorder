{{- define "audio-recorder.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "audio-recorder.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{- define "audio-recorder.labels" -}}
app.kubernetes.io/name: {{ include "audio-recorder.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | default .Chart.Version | regexReplaceAll "[^a-zA-Z0-9._-]" "-" | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}
