{{- define "zyvor.vmVolume" -}}
{{- if .Values.persistence.vmImages.hostPath }}
- name: vm-images
  hostPath:
    path: {{ .Values.persistence.vmImages.hostPath }}
    type: DirectoryOrCreate
{{- else }}
- name: vm-images
  persistentVolumeClaim:
    claimName: zyvor-image-store
{{- end }}
{{- end }}

{{- define "zyvor.vmVolumeMount" -}}
- name: vm-images
  mountPath: /var/lib/zyvor/images
{{- end }}
