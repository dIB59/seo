// src/components/analysis/atoms/ScoreRing.tsx
import { cn } from "@/src/lib/utils";

interface ScoreRingProps {
    score: number;
    size?: "sm" | "md" | "lg";
    label?: string;
}

export function ScoreRing({ score, size = "lg", label }: ScoreRingProps) {
    const dimensions = { sm: 48, md: 64, lg: 80 };
    const strokeWidth = { sm: 5, md: 6, lg: 8 };
    const dim = dimensions[size];
    const radius = (dim - strokeWidth[size]) / 2;
    const circumference = 2 * Math.PI * radius;

    return (
        <div className="relative inline-flex items-center justify-center shrink-0">
            <svg className="transform -rotate-90" width={dim} height={dim}>
                <circle cx={dim / 2} cy={dim / 2} r={radius} strokeWidth={strokeWidth[size]} stroke="currentColor" fill="none" className="text-muted/30" />
                <circle
                    cx={dim / 2} cy={dim / 2} r={radius} strokeWidth={strokeWidth[size]} stroke="currentColor" fill="none"
                    strokeDasharray={circumference}
                    strokeDashoffset={circumference - (circumference * score) / 100}
                    className={getScoreBgColor(score)}
                />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center">
                <span className={cn("font-bold", getScoreColor(score))}>{score}</span>
                {label && <span className="text-[10px] text-muted-foreground">{label}</span>}
            </div>
        </div>
    );
}