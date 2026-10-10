<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  let { title, action, children }: { title: string; action?: Snippet; children: Snippet } = $props();
  let open = $state(true);
</script>

<section class="panel">
  <header>
    <button class="panel-title" type="button" onclick={() => (open = !open)} aria-expanded={open}>
      <span>{title}</span><span class="chevron" class:open><Icon name="chevron" size={14} /></span>
    </button>
    {#if action}{@render action()}{/if}
  </header>
  {#if open}<div class="panel-content">{@render children()}</div>{/if}
</section>
