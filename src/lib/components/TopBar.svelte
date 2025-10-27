<script>
  import { Button } from "$lib/components/ui/button";

  /**
   * TopBar Component
   * Contains Import, Record, and Export buttons
   */

  let {
    onImportClick = () => {},
    onRecordClick = () => {},
    onExportClick = () => {}
  } = $props();
</script>

<header class="flex items-center justify-between h-[50px] px-4 bg-background border-b shadow-sm">
  <h1 class="text-lg font-semibold">ClipForge</h1>

  <div class="flex gap-2">
    <Button variant="outline" on:click={onImportClick} class="" disabled={false}>
      📁 Import
    </Button>
    <Button variant="outline" on:click={onRecordClick} class="" disabled={false}>
      ⭕ Record
    </Button>
    <Button variant="outline" on:click={onExportClick} class="" disabled={false}>
      💾 Export
    </Button>
  </div>
</header>
