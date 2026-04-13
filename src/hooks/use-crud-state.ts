import { useCallback, useState } from "react";
import useSWR from "swr";
import { toast } from "sonner";

/**
 * Generic CRUD state hook for settings panels. Replaces the identical
 * state + handler boilerplate in CustomChecksSettings,
 * ExtractorsSettings, and ReportPatternsSettings.
 *
 * Type params:
 *   TItem  — the full item type (has `id: string`)
 *   TForm  — the create/update params type
 */
export function useCrudState<TItem extends { id: string }, TForm>({
  swrKey,
  fetcher,
  onCreate,
  onUpdate,
  onDelete,
  entityName,
}: {
  swrKey: string;
  fetcher: () => Promise<TItem[]>;
  onCreate: (form: TForm) => Promise<TItem>;
  onUpdate: (id: string, form: TForm) => Promise<TItem>;
  onDelete: (id: string) => Promise<void>;
  entityName: string;
}) {
  const { data: items = [], mutate } = useSWR(swrKey, fetcher);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<TItem | null>(null);
  const [saving, setSaving] = useState(false);

  const openCreate = useCallback(() => {
    setEditing(null);
    setDialogOpen(true);
  }, []);

  const openEdit = useCallback((item: TItem) => {
    setEditing(item);
    setDialogOpen(true);
  }, []);

  const handleSave = useCallback(
    async (form: TForm) => {
      setSaving(true);
      try {
        if (editing) {
          const updated = await onUpdate(editing.id, form);
          mutate(
            (prev = []) => prev.map((x) => (x.id === editing.id ? updated : x)),
            false,
          );
          toast.success(`${entityName} updated`);
        } else {
          const created = await onCreate(form);
          mutate((prev = []) => [...prev, created], false);
          toast.success(`${entityName} created`);
        }
        setDialogOpen(false);
      } catch (e) {
        toast.error(e instanceof Error ? e.message : `Failed to save ${entityName.toLowerCase()}`);
      } finally {
        setSaving(false);
      }
    },
    [editing, onCreate, onUpdate, mutate, entityName],
  );

  const handleDelete = useCallback(
    async (id: string) => {
      try {
        await onDelete(id);
        mutate((prev = []) => prev.filter((x) => x.id !== id), false);
        toast.success(`${entityName} deleted`);
      } catch (e) {
        toast.error(
          e instanceof Error ? e.message : `Failed to delete ${entityName.toLowerCase()}`,
        );
      }
    },
    [onDelete, mutate, entityName],
  );

  return {
    items,
    mutate,
    dialogOpen,
    setDialogOpen,
    editing,
    saving,
    openCreate,
    openEdit,
    handleSave,
    handleDelete,
  };
}
