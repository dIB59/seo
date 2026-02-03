import { Network, Maximize2 } from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { CompleteAnalysisResult, PageAnalysisData } from "@/src/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/card";
import GraphView from "../../graph-view/GraphView";

interface SiteVisualizerProps {
    data: CompleteAnalysisResult;
    onNodeClick: (page: PageAnalysisData) => void;
}

export function SiteVisualizer({ data, onNodeClick }: SiteVisualizerProps) {
    return (
        <Card className="col-span-full lg:col-span-2 overflow-hidden">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <div className="space-y-1">
                    <CardTitle className="text-md font-medium flex items-center gap-2">
                        <Network className="h-4 w-4 text-primary" />
                        Site Architecture Map
                    </CardTitle>
                    <p className="text-xs text-muted-foreground">
                        Visualizing {data.pages.length} pages and their interconnectivity
                    </p>
                </div>
                <Button variant="outline" size="icon" className="h-8 w-8">
                    <Maximize2 className="h-4 w-4" />
                </Button>
            </CardHeader>
            <CardContent className="p-0 border-t">
                <div className="h-[500px] w-full bg-muted/10 relative">
                    <GraphView
                        data={data}
                        onNodeClick={(url: string) => {
                            const page = data.pages.find(p => p.url === url);
                            if (page) onNodeClick(page);
                        }}
                    />

                    {/* Legend Overlay */}
                    <div className="absolute bottom-4 left-4 p-2 bg-background/80 backdrop-blur border rounded-md text-[10px] space-y-1">
                        <div className="flex items-center gap-2">
                            <span className="w-2 h-2 rounded-full bg-success" /> Healthy Page
                        </div>
                        <div className="flex items-center gap-2">
                            <span className="w-2 h-2 rounded-full bg-destructive" /> Broken / Error
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}