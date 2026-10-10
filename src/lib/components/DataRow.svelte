<script lang="ts">
  import type { Snippet } from 'svelte';
  let { label, meta = '', total, leading, control }: { label: string; meta?: string; total: string | number; leading?: Snippet; control?: Snippet } = $props();
</script>

<div class="data-row">
  <div class="row-leading">{#if leading}{@render leading()}{/if}</div>
  <div class="row-label"><strong>{label}</strong>{#if meta}<span>{meta}</span>{/if}</div>
  <div class="row-control">{#if control}{@render control()}{/if}</div>
  <strong class="row-total">{total}</strong>
</div>
