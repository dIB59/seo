import { useEffect, useState } from "react";

/**
 * Synchronizes form state with an external editing value. When the
 * dialog opens, the form is reset to either the editing item's
 * values or the default empty state.
 *
 * Replaces the repeated pattern:
 *   useEffect(() => { if (open) setForm(editing ? paramsFrom(editing) : EMPTY); }, [open, editing]);
 *
 * Usage:
 *   const [form, setForm] = useFormSync(open, editing, EMPTY_PARAMS, paramsFrom);
 */
export function useFormSync<TItem, TForm>(
  open: boolean,
  editing: TItem | null,
  defaults: TForm,
  toForm: (item: TItem) => TForm,
): [TForm, React.Dispatch<React.SetStateAction<TForm>>] {
  const [form, setForm] = useState<TForm>(defaults);

  useEffect(() => {
    if (!open) return;
    setForm(editing ? toForm(editing) : defaults);
  }, [open, editing, defaults, toForm]);

  return [form, setForm];
}
