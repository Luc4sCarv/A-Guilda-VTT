<script lang="ts">
  import Icon from './Icon.svelte';
  let { label = 'Rolar dado' }: { label?: string } = $props();
</script>

<button class="dice-button" type="button" aria-label={label} title={`${label} — integração com o core será adicionada na próxima etapa`}><Icon name="dice" size={16} /></button>
