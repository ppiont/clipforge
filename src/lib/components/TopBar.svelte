<script>
  import { Button } from "$lib/components/ui/button";

  /**
   * TopBar Component
   * Contains Import, Record, and Export buttons
   */

  export let onImportClick = () => {};
  export let onRecordClick = () => {};
  export let onExportClick = () => {};
</script>

<header class="flex items-center justify-between h-[50px] px-4 bg-background border-b shadow-sm">
  <h1 class="text-lg font-semibold">ClipForge</h1>

  <div class="flex gap-2">
    <Button variant="outline" on:click={onImportClick}>
      📁 Import
    </Button>
    <Button variant="outline" on:click={onRecordClick}>
      ⭕ Record
    </Button>
    <Button variant="outline" on:click={onExportClick}>
      💾 Export
    </Button>
  </div>
</header>
