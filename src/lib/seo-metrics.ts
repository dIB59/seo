export function getScoreColor(score: number) {
    if (score >= 80) return "text-success";
    if (score >= 50) return "text-warning";
    return "text-destructive";
}

export function getScoreBgColor(score: number) {
    if (score >= 80) return "bg-success";
    if (score >= 50) return "bg-warning";
    return "bg-destructive";
}

export function getScoreLabel(score: number) {
    if (score >= 90) return "Excellent";
    if (score >= 80) return "Good";
    if (score >= 60) return "Fair";
    if (score >= 40) return "Poor";
    return "Critical";
}

export function getLoadTimeColor(time: number) {
    if (time < 1) return "text-success";
    if (time < 2) return "text-warning";
    return "text-destructive";
}

