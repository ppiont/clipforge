<script>
  import { Button } from "$lib/components/ui/button";
  import { FolderOpen, Circle, Save } from "@lucide/svelte";

  /**
   * TopBar Component
   * Contains Import, Record, and Export buttons
   */

  let {
    onImportClick = () => {},
    onRecordClick = () => {},
    onExportClick = () => {},
  } = $props();
</script>

<header class="flex items-center justify-between h-[50px] px-4 bg-background border-b shadow-sm">
  <h1 class="text-lg font-semibold">ClipForge</h1>

  <div class="flex gap-2">
    <Button
      class=""
      variant="outline"
      disabled={false}
      onclick={onImportClick}
    >
      <FolderOpen class="w-4 h-4" />
      Import
    </Button>
    <Button
      class=""
      variant="outline"
      disabled={false}
      onclick={onRecordClick}
    >
      <Circle class="w-4 h-4 text-destructive" />
      Record
    </Button>
    <Button
      class=""
      variant="outline"
      disabled={false}
      onclick={onExportClick}
    >
      <Save class="w-4 h-4" />
      Export
    </Button>
  </div>
</header>
