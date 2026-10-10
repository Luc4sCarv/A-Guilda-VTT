<script lang="ts">
  let { label, value, detail = '' }: { label: string; value: number; detail?: string } = $props();
</script>

<div class="stat-box" title={detail || label}>
  <strong>{value >= 0 ? `+${value}` : value}</strong>
  <span>{label}</span>
</div>
