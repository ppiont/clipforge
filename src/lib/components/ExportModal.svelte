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

  /** @param {boolean} _value */
  function handleOpenChange(_value) {
    onClose();
  }

  /** @param {string} value */
  function updateResolution(value) {
    resolution = value;
  }
</script>

<Dialog open={show} onOpenChange={handleOpenChange}>
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
          <div class="space-y-2">
            <label class="flex items-center gap-2">
              <input
                type="radio"
                name="resolution"
                value="Source"
                checked={resolution === "Source"}
                onchange={(e) => updateResolution(e.currentTarget.value)}
              />
              <span>Source (Original)</span>
            </label>
            <label class="flex items-center gap-2">
              <input
                type="radio"
                name="resolution"
                value="720p"
                checked={resolution === "720p"}
                onchange={(e) => updateResolution(e.currentTarget.value)}
              />
              <span>720p</span>
            </label>
            <label class="flex items-center gap-2">
              <input
                type="radio"
                name="resolution"
                value="1080p"
                checked={resolution === "1080p"}
                onchange={(e) => updateResolution(e.currentTarget.value)}
              />
              <span>1080p</span>
            </label>
          </div>
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
      <Button variant="outline" disabled={isExporting} class="" onclick={onClose}>
        Cancel
      </Button>
      <Button disabled={isExporting || !show} class="">
        {isExporting ? 'Exporting...' : 'Export'}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
