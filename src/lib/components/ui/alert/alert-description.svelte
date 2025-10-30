<script>
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	} = $props();
</script>

<div
	bind:this={ref}
	class={cn("text-sm [&_p]:leading-relaxed", className)}
	{...restProps}
>
	{@render children?.()}
</div>
