<script>
  import '../app.css';
</script>

<slot />

<style global>
  :global(html, body) {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  :global(body > div) {
    width: 100%;
    height: 100%;
  }

  ::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  ::-webkit-scrollbar-track {
    background: hsl(var(--muted));
  }

  ::-webkit-scrollbar-thumb {
    background: hsl(var(--muted-foreground));
    border-radius: 4px;
  }

  ::-webkit-scrollbar-thumb:hover {
    background: hsl(var(--foreground));
  }
</style>
