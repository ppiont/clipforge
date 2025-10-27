<script>
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogFooter
  } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "$lib/components/ui/select";
  import { Progress } from "$lib/components/ui/progress";

  /**
   * ExportModal Component
   * Modal dialog for exporting video
   * (Stub for Task 6.2)
   */

  export let show = false;
  export let onClose = () => {};

  let resolution = "Source";
  let isExporting = false;
  let progress = 0;
</script>

<Dialog open={show} onOpenChange={onClose}>
  <DialogContent class="sm:max-w-[500px]">
    <DialogHeader>
      <DialogTitle>Export Video</DialogTitle>
      <DialogDescription>
        Configure and export your video composition
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      {#if !isExporting}
        <div class="space-y-2">
          <label for="resolution" class="text-sm font-medium">Resolution</label>
          <Select bind:value={resolution}>
            <SelectTrigger id="resolution">
              <SelectValue placeholder="Select resolution" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="Source">Source</SelectItem>
              <SelectItem value="720p">720p</SelectItem>
              <SelectItem value="1080p">1080p</SelectItem>
            </SelectContent>
          </Select>
        </div>
      {:else}
        <div class="space-y-2">
          <div class="flex items-center justify-between text-sm">
            <span>Exporting...</span>
            <span class="font-medium">{progress}%</span>
          </div>
          <Progress value={progress} class="w-full" />
        </div>
      {/if}
    </div>

    <DialogFooter>
      <Button variant="outline" on:click={onClose} disabled={isExporting}>
        Cancel
      </Button>
      <Button disabled={isExporting || !show}>
        {isExporting ? 'Exporting...' : 'Export'}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
