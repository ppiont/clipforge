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
  <h1 class="text-xl font-bold text-foreground">ClipForge</h1>

  <div class="flex gap-2">
    <Button variant="outline" onclick={onImportClick}>
      <FolderOpen />
      Import
    </Button>
    <Button variant="outline" onclick={onRecordClick}>
      <Circle class="fill-destructive text-destructive" />
      Record
    </Button>
    <Button variant="default" onclick={onExportClick}>
      <Save />
      Export
    </Button>
  </div>
</header>
