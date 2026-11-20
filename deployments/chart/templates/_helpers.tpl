{{- define "chart.fullname" -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end }}

{{- define "chart.labels" -}}
app.kubernetes.io/name: giong
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
