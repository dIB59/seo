"use client";

import { Component, type ErrorInfo, type ReactNode } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "@/src/components/ui/button";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

/**
 * React error boundary that catches render errors in child components
 * and shows a recovery UI instead of crashing the entire app.
 *
 * Usage:
 *   <ErrorBoundary>
 *     <AnalysisDashboard />
 *   </ErrorBoundary>
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("[ErrorBoundary] Caught error:", error, info.componentStack);
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <div className="flex flex-col items-center justify-center min-h-[300px] gap-4 p-8">
          <div className="rounded-full bg-destructive/10 p-3">
            <AlertTriangle className="h-6 w-6 text-destructive" />
          </div>
          <div className="text-center space-y-1">
            <h3 className="font-semibold">Something went wrong</h3>
            <p className="text-sm text-muted-foreground max-w-md">
              {this.state.error?.message || "An unexpected error occurred."}
            </p>
          </div>
          <Button variant="outline" size="sm" onClick={this.handleRetry}>
            <RefreshCw className="h-3.5 w-3.5 mr-1.5" />
            Try again
          </Button>
        </div>
      );
    }

    return this.props.children;
  }
}
