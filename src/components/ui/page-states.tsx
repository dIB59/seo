"use client";

import { Loader2 } from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { ArrowLeft } from "lucide-react";

interface LoadingStateProps {
  message?: string;
}

/**
 * Full-screen centered loading spinner with an optional message.
 * Use this in page-level loading states.
 */
export function LoadingState({ message = "Loading..." }: LoadingStateProps) {
  return (
    <div className="flex flex-col items-center justify-center min-h-screen">
      <Loader2 className="h-8 w-8 animate-spin text-primary" />
      <p className="mt-4 text-muted-foreground">{message}</p>
    </div>
  );
}

interface ErrorStateProps {
  title?: string;
  description?: string;
  backLabel?: string;
  onBack?: () => void;
}

/**
 * Full-screen centered error state with an optional back button.
 * Use this in page-level error states.
 */
export function ErrorState({
  title = "Something went wrong",
  description = "An unexpected error occurred.",
  backLabel = "Go Back",
  onBack,
}: ErrorStateProps) {
  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-4 text-center">
      <h1 className="text-2xl font-bold text-destructive mb-2">{title}</h1>
      <p className="text-muted-foreground mb-4">{description}</p>
      {onBack && (
        <Button onClick={onBack}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          {backLabel}
        </Button>
      )}
    </div>
  );
}
