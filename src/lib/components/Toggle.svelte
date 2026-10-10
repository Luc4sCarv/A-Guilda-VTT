<script lang="ts">
  let { checked = $bindable(), label }: { checked: boolean; label: string } = $props();
</script>

<label class="toggle" title={label}>
  <input type="checkbox" bind:checked />
  <span aria-hidden="true">✓</span>
  <span class="sr-only">{label}</span>
</label>
