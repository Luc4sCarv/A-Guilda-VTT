<script lang="ts">
  let { children }: { children: import('svelte').Snippet } = $props();
</script>

<span class="tag">{@render children()}</span>
