<script>
  import '../app.css';
</script>

<slot />

<style>
  :global(html, body) {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  :global(body > div) {
    width: 100%;
    height: 100%;
  }
</style>
