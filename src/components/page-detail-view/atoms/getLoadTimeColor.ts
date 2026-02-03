export default function getLoadTimeColor(time: number) {
    if (time < 1) return "text-success"
    if (time < 2) return "text-warning"
    return "text-destructive"
}
