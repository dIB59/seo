import { useRef, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

type UseVirtualizerRefArgs = { count: number; estimateSize?: number };

export function useVirtualizerRef({ count, estimateSize = 53 }: UseVirtualizerRefArgs) {
  const parentRef = useRef<HTMLElement | null>(null);
  const roRef = useRef<ResizeObserver | null>(null);

  const virtualizer = useVirtualizer({
    count,
    getScrollElement: () => parentRef.current,
    estimateSize: () => estimateSize,
  });

  const setRef = useCallback(
    (el: HTMLElement | null) => {
      parentRef.current = el;

      if (el) {
        requestAnimationFrame(() => virtualizer.measure());

        const ro = new ResizeObserver(() => virtualizer.measure());
        ro.observe(el);
        roRef.current = ro;
      }
    },
    [virtualizer],
  );

  const measure = useCallback(() => virtualizer.measure(), [virtualizer]);

  return {
    parentRef,
    setRef,
    virtualItems: virtualizer.getVirtualItems(),
    totalSize: virtualizer.getTotalSize(),
    measure,
  } as const;
}
