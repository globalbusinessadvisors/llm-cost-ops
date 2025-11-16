{{/*
Expand the name of the chart.
*/}}
{{- define "llm-cost-ops.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "llm-cost-ops.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "llm-cost-ops.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "llm-cost-ops.labels" -}}
helm.sh/chart: {{ include "llm-cost-ops.chart" . }}
{{ include "llm-cost-ops.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- with .Values.podLabels }}
{{ toYaml . }}
{{- end }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "llm-cost-ops.selectorLabels" -}}
app.kubernetes.io/name: {{ include "llm-cost-ops.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "llm-cost-ops.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "llm-cost-ops.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Return the proper image name
*/}}
{{- define "llm-cost-ops.image" -}}
{{- $registry := .Values.image.registry -}}
{{- if .Values.global }}
  {{- if .Values.global.imageRegistry }}
    {{- $registry = .Values.global.imageRegistry -}}
  {{- end -}}
{{- end -}}
{{- $repository := .Values.image.repository -}}
{{- $tag := .Values.image.tag | default .Chart.AppVersion -}}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $repository $tag -}}
{{- else }}
{{- printf "%s:%s" $repository $tag -}}
{{- end }}
{{- end }}

{{/*
Return the proper PostgreSQL image name
*/}}
{{- define "llm-cost-ops.postgresql.image" -}}
{{- $registry := .Values.postgresql.image.registry -}}
{{- if .Values.global }}
  {{- if .Values.global.imageRegistry }}
    {{- $registry = .Values.global.imageRegistry -}}
  {{- end -}}
{{- end -}}
{{- $repository := .Values.postgresql.image.repository -}}
{{- $tag := .Values.postgresql.image.tag -}}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $repository $tag -}}
{{- else }}
{{- printf "%s:%s" $repository $tag -}}
{{- end }}
{{- end }}

{{/*
Return the proper Redis image name
*/}}
{{- define "llm-cost-ops.redis.image" -}}
{{- $registry := .Values.redis.image.registry -}}
{{- if .Values.global }}
  {{- if .Values.global.imageRegistry }}
    {{- $registry = .Values.global.imageRegistry -}}
  {{- end -}}
{{- end -}}
{{- $repository := .Values.redis.image.repository -}}
{{- $tag := .Values.redis.image.tag -}}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $repository $tag -}}
{{- else }}
{{- printf "%s:%s" $repository $tag -}}
{{- end }}
{{- end }}

{{/*
Return the proper NATS image name
*/}}
{{- define "llm-cost-ops.nats.image" -}}
{{- $registry := .Values.nats.image.registry -}}
{{- if .Values.global }}
  {{- if .Values.global.imageRegistry }}
    {{- $registry = .Values.global.imageRegistry -}}
  {{- end -}}
{{- end -}}
{{- $repository := .Values.nats.image.repository -}}
{{- $tag := .Values.nats.image.tag -}}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $repository $tag -}}
{{- else }}
{{- printf "%s:%s" $repository $tag -}}
{{- end }}
{{- end }}

{{/*
Return image pull secrets
*/}}
{{- define "llm-cost-ops.imagePullSecrets" -}}
{{- $pullSecrets := list }}
{{- if .Values.global }}
  {{- if .Values.global.imagePullSecrets }}
    {{- range .Values.global.imagePullSecrets }}
      {{- $pullSecrets = append $pullSecrets . }}
    {{- end }}
  {{- end }}
{{- end }}
{{- range .Values.imagePullSecrets }}
  {{- $pullSecrets = append $pullSecrets . }}
{{- end }}
{{- if (not (empty $pullSecrets)) }}
imagePullSecrets:
{{- range $pullSecrets }}
  - name: {{ . }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Return the database URL
*/}}
{{- define "llm-cost-ops.databaseUrl" -}}
{{- if eq .Values.config.database.type "sqlite" -}}
sqlite://{{ .Values.config.database.sqlitePath }}
{{- else if eq .Values.config.database.type "postgres" -}}
{{- if .Values.config.database.existingSecret -}}
postgresql://{{ .Values.config.database.user }}:$(DB_PASSWORD)@{{ .Values.config.database.host }}:{{ .Values.config.database.port }}/{{ .Values.config.database.name }}?sslmode={{ .Values.config.database.sslMode }}
{{- else -}}
postgresql://{{ .Values.config.database.user }}:{{ .Values.config.database.password }}@{{ .Values.config.database.host }}:{{ .Values.config.database.port }}/{{ .Values.config.database.name }}?sslmode={{ .Values.config.database.sslMode }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Return the JWT secret
*/}}
{{- define "llm-cost-ops.jwtSecret" -}}
{{- if .Values.config.auth.jwt.existingSecret -}}
$(JWT_SECRET)
{{- else -}}
{{ .Values.config.auth.jwt.secret }}
{{- end }}
{{- end }}

{{/*
Return the storage class for persistence
*/}}
{{- define "llm-cost-ops.storageClass" -}}
{{- $storageClass := .Values.persistence.storageClass -}}
{{- if .Values.global }}
  {{- if .Values.global.storageClass }}
    {{- $storageClass = .Values.global.storageClass -}}
  {{- end -}}
{{- end -}}
{{- if $storageClass }}
storageClassName: {{ $storageClass }}
{{- end }}
{{- end }}

{{/*
PostgreSQL fullname
*/}}
{{- define "llm-cost-ops.postgresql.fullname" -}}
{{- printf "%s-postgres" (include "llm-cost-ops.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Redis fullname
*/}}
{{- define "llm-cost-ops.redis.fullname" -}}
{{- printf "%s-redis" (include "llm-cost-ops.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
NATS fullname
*/}}
{{- define "llm-cost-ops.nats.fullname" -}}
{{- printf "%s-nats" (include "llm-cost-ops.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
ConfigMap name
*/}}
{{- define "llm-cost-ops.configMapName" -}}
{{- printf "%s-config" (include "llm-cost-ops.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Secret name
*/}}
{{- define "llm-cost-ops.secretName" -}}
{{- printf "%s-secret" (include "llm-cost-ops.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Return true if a ConfigMap object should be created
*/}}
{{- define "llm-cost-ops.createConfigMap" -}}
{{- if .Values.configMap.create }}
    {{- true -}}
{{- end -}}
{{- end -}}

{{/*
Return true if a Secret object should be created
*/}}
{{- define "llm-cost-ops.createSecret" -}}
{{- if .Values.secret.create }}
    {{- true -}}
{{- end -}}
{{- end -}}

{{/*
Compile all warnings into a single message.
*/}}
{{- define "llm-cost-ops.validateValues" -}}
{{- $messages := list -}}
{{- $messages := append $messages (include "llm-cost-ops.validateValues.database" .) -}}
{{- $messages := append $messages (include "llm-cost-ops.validateValues.auth" .) -}}
{{- $messages := without $messages "" -}}
{{- $message := join "\n" $messages -}}
{{- if $message -}}
{{-   printf "\nVALUES VALIDATION:\n%s" $message -}}
{{- end -}}
{{- end -}}

{{/*
Validate database configuration
*/}}
{{- define "llm-cost-ops.validateValues.database" -}}
{{- if and (eq .Values.config.database.type "postgres") (not .Values.postgresql.enabled) (not .Values.config.database.host) -}}
llm-cost-ops: database
    PostgreSQL is selected but not enabled and no external host is configured.
    Either enable PostgreSQL (postgresql.enabled=true) or provide an external host.
{{- end -}}
{{- end -}}

{{/*
Validate authentication configuration
*/}}
{{- define "llm-cost-ops.validateValues.auth" -}}
{{- if and .Values.config.auth.enabled (not .Values.config.auth.jwt.existingSecret) (eq .Values.config.auth.jwt.secret "change-me-in-production") -}}
llm-cost-ops: auth
    Authentication is enabled but JWT secret is set to default value.
    Please provide a secure JWT secret or use existingSecret.
{{- end -}}
{{- end -}}
