<script lang="ts">
  let {
    label,
    current = $bindable(),
    maximum,
    temporary = 0,
    tone,
  }: { label: string; current: number; maximum: number; temporary?: number; tone: 'vida' | 'mana' } = $props();

  let percentage = $derived(Math.max(0, Math.min(100, (current / maximum) * 100)));
</script>

<div class="resource" style={`--resource: var(--${tone})`}>
  <div class="resource-heading"><strong>{label}</strong><span>{current} / {maximum}</span></div>
  <div class="resource-track" aria-label={`${label}: ${current} de ${maximum}`} role="meter" aria-valuenow={current} aria-valuemin="0" aria-valuemax={maximum}>
    <span style={`width: ${percentage}%`}></span>
  </div>
  <div class="resource-fields">
    <label>Atual <input type="number" bind:value={current} min="0" max={maximum} /></label>
    <span>Máximo <b>{maximum}</b></span>
    <span>Temporário <b>{temporary}</b></span>
  </div>
</div>
