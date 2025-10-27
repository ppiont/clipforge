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
  import { Select, SelectContent, SelectItem, SelectTrigger } from "$lib/components/ui/select";
  import { Progress } from "$lib/components/ui/progress";

  /**
   * ExportModal Component
   * Modal dialog for exporting video
   * (Stub for Task 6.2)
   */

  let {
    show = $bindable(false),
    onClose = () => {}
  } = $props();

  let resolution = $state("Source");
  let isExporting = $state(false);
  let progress = $state(0);
</script>

<Dialog open={show} onOpenChange={onClose}>
  <DialogContent class="sm:max-w-[500px]" portalProps={{}}>
    <DialogHeader class="">
      <DialogTitle class="">Export Video</DialogTitle>
      <DialogDescription class="">
        Configure and export your video composition
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      {#if !isExporting}
        <div class="space-y-2">
          <label for="resolution" class="text-sm font-medium">Resolution</label>
          <Select bind:value={resolution}>
            <SelectTrigger id="resolution" class="">
              <span>{resolution || 'Select resolution'}</span>
            </SelectTrigger>
            <SelectContent class="" portalProps={{}}>
              <SelectItem value="Source" label="">Source</SelectItem>
              <SelectItem value="720p" label="">720p</SelectItem>
              <SelectItem value="1080p" label="">1080p</SelectItem>
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

    <DialogFooter class="">
      <Button variant="outline" on:click={onClose} disabled={isExporting} class="">
        Cancel
      </Button>
      <Button disabled={isExporting || !show} class="">
        {isExporting ? 'Exporting...' : 'Export'}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
