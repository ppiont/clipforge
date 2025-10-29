<script>
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	} = $props();
</script>

<h5
	bind:this={ref}
	class={cn("mb-1 font-medium leading-none tracking-tight", className)}
	{...restProps}
>
	{@render children?.()}
</h5>
