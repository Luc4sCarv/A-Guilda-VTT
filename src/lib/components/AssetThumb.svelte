<script lang="ts">
  import { onMount } from 'svelte';
  import { assetBlobUrl } from '../mesa';
  let { id, alt='' }: {id:string;alt?:string}=$props();
  let url=$state('');
  onMount(()=>{let alive=true;void assetBlobUrl(id).then(value=>{if(alive)url=value;else URL.revokeObjectURL(value);}).catch(()=>{});return ()=>{alive=false;if(url)URL.revokeObjectURL(url);};});
</script>
{#if url}<img class="mesa-asset-thumb" src={url} {alt}/>{:else}<span class="mesa-asset-placeholder" aria-hidden="true">◇</span>{/if}
