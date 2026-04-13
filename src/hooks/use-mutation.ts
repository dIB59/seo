import { useCallback, useState } from "react";
import { toast } from "sonner";

interface UseMutationOptions<T> {
  onSuccess?: (data: T) => void;
  successMessage?: string;
  errorMessage?: string;
}

/**
 * Wraps an async API call with loading state and toast notifications.
 * Replaces the `setSaving(true) → try/catch → toast → finally` pattern
 * repeated 12+ times across config components.
 *
 * Usage:
 *   const save = useMutation(updateTemplate, { successMessage: "Saved" });
 *   <Button onClick={() => save.execute(template)} disabled={save.isLoading} />
 */
export function useMutation<TArgs extends unknown[], TResult>(
  fn: (...args: TArgs) => Promise<TResult>,
  options: UseMutationOptions<TResult> = {},
) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(
    async (...args: TArgs): Promise<TResult | undefined> => {
      setIsLoading(true);
      setError(null);
      try {
        const result = await fn(...args);
        if (options.successMessage) {
          toast.success(options.successMessage);
        }
        options.onSuccess?.(result);
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        toast.error(options.errorMessage ?? msg);
        return undefined;
      } finally {
        setIsLoading(false);
      }
    },
    [fn, options],
  );

  return { execute, isLoading, error };
}
