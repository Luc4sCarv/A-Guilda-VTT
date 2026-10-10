<script lang="ts">
  let { name, size = 18 }: { name: string; size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  {#if name === 'users'}<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75" />
  {:else if name === 'home'}<path d="m3 11 9-8 9 8v10h-6v-6H9v6H3V11Z" />
  {:else if name === 'book'}<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20V3H6.5A2.5 2.5 0 0 0 4 5.5v14ZM8 7h8M8 11h6" />
  {:else if name === 'table'}<rect x="3" y="4" width="18" height="16" rx="1" /><path d="M3 9h18M9 9v11" />
  {:else if name === 'cursor'}<path d="m4 3 1 17 4.5-4.5 3.6 5.5 2.5-1.6-3.6-5.4 6-1L4 3Z" />
  {:else if name === 'hand'}<path d="M7 13V6a1.5 1.5 0 0 1 3 0v5-7a1.5 1.5 0 0 1 3 0v7-5a1.5 1.5 0 0 1 3 0v6-3a1.5 1.5 0 0 1 3 0v7c0 4-3 6-7 6h-1c-2 0-3.5-1-5-3l-3-4a1.5 1.5 0 0 1 2.5-1.7L7 13Z" />
  {:else if name === 'pen'}<path d="m4 20 4.5-1 11-11a2.1 2.1 0 0 0-3-3l-11 11L4 20ZM14.5 7.5l3 3" />
  {:else if name === 'text'}<path d="M4 5h16M12 5v15M8 20h8M4 5v3M20 5v3" />
  {:else if name === 'ruler'}<path d="m4 17 13-13 3 3L7 20l-3-3ZM11 10l2 2M14 7l2 2M8 13l2 2" />
  {:else if name === 'turn'}<path d="M4 6h13M4 11h13M4 16h9M18 15l3 3-3 3M21 18h-6" />
  {:else if name === 'pin'}<path d="M12 22s7-7 7-13a7 7 0 1 0-14 0c0 6 7 13 7 13Z"/><circle cx="12" cy="9" r="2" />
  {:else if name === 'fog'}<path d="M2 8h12a3 3 0 1 0-3-3M2 12h18a2 2 0 1 1-2 2M2 16h10a3 3 0 1 1-3 3" />
  {:else if name === 'chart'}<path d="M4 19V9M10 19V5M16 19v-7M22 19V3" />
  {:else if name === 'plus'}<path d="M12 5v14M5 12h14" />
  {:else if name === 'sun'}<circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.93 4.93l1.42 1.42M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.42-1.42M17.66 6.34l1.41-1.41" />
  {:else if name === 'moon'}<path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8Z" />
  {:else if name === 'settings'}<circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.88l.06.06-2.83 2.83-.06-.06a1.7 1.7 0 0 0-1.88-.34 1.7 1.7 0 0 0-1.03 1.56V21h-4v-.08A1.7 1.7 0 0 0 9 19.37a1.7 1.7 0 0 0-1.88.34l-.06.06-2.83-2.83.06-.06A1.7 1.7 0 0 0 4.63 15a1.7 1.7 0 0 0-1.55-1.03H3v-4h.08A1.7 1.7 0 0 0 4.63 9a1.7 1.7 0 0 0-.34-1.88l-.06-.06 2.83-2.83.06.06A1.7 1.7 0 0 0 9 4.63a1.7 1.7 0 0 0 1.03-1.55V3h4v.08A1.7 1.7 0 0 0 15 4.63a1.7 1.7 0 0 0 1.88-.34l.06-.06 2.83 2.83-.06.06A1.7 1.7 0 0 0 19.37 9a1.7 1.7 0 0 0 1.55 1.03H21v4h-.08A1.7 1.7 0 0 0 19.4 15Z" />
  {:else if name === 'tune'}<path d="M4 7h16M4 17h16"/><circle cx="9" cy="7" r="2" fill="currentColor" stroke="none"/><circle cx="15" cy="17" r="2" fill="currentColor" stroke="none"/>
  {:else if name === 'back'}<path d="m15 18-6-6 6-6" />
  {:else if name === 'download'}<path d="M12 3v12M7 10l5 5 5-5M5 21h14" />
  {:else if name === 'save'}<path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2Z" /><path d="M17 21v-8H7v8M7 3v5h8" />
  {:else if name === 'sword'}<path d="m14.5 4.5 5-2-2 5L9 16l-3-3 8.5-8.5ZM5 14l-2 2 5 5 2-2M14 8l2 2" />
  {:else if name === 'dice'}<path d="m12 2 8.7 5v10L12 22l-8.7-5V7L12 2Z" /><circle cx="12" cy="12" r="1" /><circle cx="8" cy="9" r=".7" /><circle cx="16" cy="15" r=".7" />
  {:else if name === 'heart'}<path d="M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z" />
  {:else if name === 'spark'}<path d="m12 2 1.7 6.3L20 10l-6.3 1.7L12 18l-1.7-6.3L4 10l6.3-1.7L12 2ZM19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8L19 16Z" />
  {:else if name === 'power'}<path d="M12 2v10M6.3 4.8a8 8 0 1 0 11.4 0" />
  {:else if name === 'bag'}<path d="M6 8h12l1 13H5L6 8ZM9 8V6a3 3 0 0 1 6 0v2" />
  {:else if name === 'close'}<path d="m6 6 12 12M18 6 6 18" />
  {:else if name === 'guide'}<path d="M3 5.5A2.5 2.5 0 0 1 5.5 3H11v16H5.5A2.5 2.5 0 0 0 3 21V5.5ZM21 5.5A2.5 2.5 0 0 0 18.5 3H13v16h5.5A2.5 2.5 0 0 1 21 21V5.5Z" />
  {:else if name === 'image'}<rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="8" cy="9" r="1.5"/><path d="m4 18 5-5 3 3 3-4 5 6"/>
  {:else if name === 'music'}<path d="M9 18V5l11-2v13M9 8l11-2"/><circle cx="6" cy="18" r="3"/><circle cx="17" cy="16" r="3"/>
  {:else if name === 'chevron'}<path d="m9 18 6-6-6-6" />
  {:else if name === 'trash'}<path d="M4 7h16M9 7V4h6v3M7 7l1 14h8l1-14M10 11v6M14 11v6" />
  {:else if name === 'person'}<circle cx="12" cy="7" r="4"/><path d="M4 22a8 8 0 0 1 16 0"/>
  {:else if name === 'dna'}<path d="M7 3c0 6 10 12 10 18M17 3C17 9 7 15 7 21M8 6h8M8 18h8M10 10h4M10 14h4"/>
  {:else if name === 'devotion'}<path d="M12 2 5 6v6c0 5 3 8 7 10 4-2 7-5 7-10V6l-7-4Z"/><path d="M12 7v9M9 10h6"/>
  {:else if name === 'skull'}<path d="M5 11a7 7 0 1 1 14 0c0 3-1 5-3 6v4H8v-4c-2-1-3-3-3-6Z"/><circle cx="9" cy="11" r="1"/><circle cx="15" cy="11" r="1"/><path d="M10 17h4"/>
  {:else if name === 'handshake'}<path d="m8 12 3 3a2 2 0 0 0 3 0l4-4M3 8l5-3 4 3-4 5-5-2V8ZM21 8l-5-3-4 3"/><path d="m15 16-2 2-1-1-2 2-5-5"/>
  {:else}<circle cx="12" cy="12" r="9" />{/if}
</svg>
