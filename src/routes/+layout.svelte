<script>
  /**
   * Root Layout
   * Global styling and setup for ClipForge
   */
</script>

<slot />

<style global>
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  html,
  body {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto',
      'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans',
      'Helvetica Neue', sans-serif;
    font-size: 14px;
    line-height: 1.5;
    color: #333;
    background: #fff;
  }

  /* SvelteKit default app wrapper */
  :global(body) {
    margin: 0;
    padding: 0;
  }

  :global(#svelte) {
    width: 100%;
    height: 100%;
  }

  /* Scrollbar styling */
  ::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  ::-webkit-scrollbar-track {
    background: #f5f5f5;
  }

  ::-webkit-scrollbar-thumb {
    background: #ccc;
    border-radius: 4px;
  }

  ::-webkit-scrollbar-thumb:hover {
    background: #999;
  }

  /* Selection styling */
  ::selection {
    background: #1976d2;
    color: white;
  }
</style>
