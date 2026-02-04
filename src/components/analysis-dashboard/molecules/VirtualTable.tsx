// src/components/ui/VirtualTable.tsx
import { useRef, ReactNode } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

interface VirtualTableProps<T> {
  data: T[];
  renderHeader: () => ReactNode;
  renderRow: (item: T, index: number) => ReactNode;
  estimateRowHeight?: number;
  overscan?: number;
  maxHeight?: string;
  className?: string;
  gridTemplateColumns?: string;
}

export function VirtualTable<T>({
  data,
  renderHeader,
  renderRow,
  estimateRowHeight = 53,
  overscan = 10,
  maxHeight = "calc(100vh - 200px)",
  className = "",
  gridTemplateColumns = "1fr 80px 80px 100px 80px 80px 80px 80px 40px",
}: VirtualTableProps<T>) {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: data.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => estimateRowHeight,
    overscan,
  });

  return (
    <div ref={parentRef} className={`overflow-auto ${className}`} style={{ maxHeight }}>
      <div className="w-full" style={{ display: 'grid', gridTemplateColumns }}>
        {/* Header */}
        <div className="sticky top-0 bg-background z-10" style={{ 
          display: 'contents',
        }}>
          {renderHeader()}
        </div>
        
        {/* Virtual rows container */}
        <div style={{ 
          display: 'contents',
          position: 'relative',
        }}>
          <div style={{ 
            gridColumn: '1 / -1',
            height: `${virtualizer.getTotalSize()}px`,
            position: 'relative',
          }}>
            {virtualizer.getVirtualItems().map((virtualRow) => (
              <div
                key={virtualRow.index}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  transform: `translateY(${virtualRow.start}px)`,
                  display: 'grid',
                  gridTemplateColumns,
                }}
              >
                {renderRow(data[virtualRow.index], virtualRow.index)}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}