import { SeoIssue } from "@/src/lib/types"
import { Accordion, AccordionItem, AccordionTrigger, AccordionContent } from "@radix-ui/react-accordion"
import { CheckCircle2 } from "lucide-react"
import { Card, CardContent } from "../../ui/card"
import { IssueBadge } from "../atoms/IssueBadge"
import { IssueIcon } from "../atoms/IssueIcon"

export function IssuesAccordion({ issues }: { issues: SeoIssue[] }) {
    const groupedIssues: Record<string, SeoIssue[]> = {}
    issues.forEach((issue) => {
        if (!groupedIssues[issue.title]) groupedIssues[issue.title] = []
        groupedIssues[issue.title].push(issue)
    })

    if (Object.keys(groupedIssues).length === 0) {
        return (
            <Card>
                <CardContent className="p-6 text-center">
                    <CheckCircle2 className="h-12 w-12 text-success mx-auto mb-2" />
                    <p className="text-muted-foreground">No issues found. Great job!</p>
                </CardContent>
            </Card>
        )
    }

    return (
        <Accordion type="multiple" className="space-y-2">
            {Object.entries(groupedIssues).map(([title, issueGroup]) => (
                <AccordionItem key={title} value={title} className="border rounded-lg px-4">
                    <AccordionTrigger className="hover:no-underline">
                        <div className="flex items-center gap-3">
                            <IssueIcon type={issueGroup[0].issue_type} />
                            <span className="font-medium">{title}</span>
                            <IssueBadge type={issueGroup[0].issue_type} />
                            <span className="text-xs text-muted-foreground">
                                {issueGroup.length} {issueGroup.length === 1 ? "page" : "pages"}
                            </span>
                        </div>
                    </AccordionTrigger>
                    <AccordionContent>
                        <div className="space-y-3 pt-2">
                            <p className="text-sm text-muted-foreground">{issueGroup[0].description}</p>
                            <div className="p-3 bg-muted/50 rounded-lg">
                                <p className="text-sm font-medium mb-1">Recommendation</p>
                                <p className="text-sm text-muted-foreground">{issueGroup[0].recommendation}</p>
                            </div>
                            <div className="space-y-1">
                                <p className="text-xs font-medium text-muted-foreground">Affected Pages:</p>
                                {issueGroup.map((issue, idx) => (
                                    <p key={idx} className="text-xs text-muted-foreground truncate">
                                        {issue.page_url}
                                    </p>
                                ))}
                            </div>
                        </div>
                    </AccordionContent>
                </AccordionItem>
            ))}
        </Accordion>
    )
}