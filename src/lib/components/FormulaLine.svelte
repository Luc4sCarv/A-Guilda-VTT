<script lang="ts">
  let { label, terms, total }: { label: string; terms: string[]; total: number } = $props();
</script>

<div class="formula">
  <span>{label}</span>
  <div>{#each terms as term, index}<span>{term}</span>{#if index < terms.length - 1}<i>+</i>{/if}{/each}<b>= {total}</b></div>
</div>
