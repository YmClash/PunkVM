import React from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { BarChart, Bar, LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

const PipelineProgress = () => {
    const currentMetrics = [
        {
            phase: "Initial Pipeline",
            ipc: 0.5,
            hazards: 8,
            branchMispredicts: 12,
            completion: 100
        },
        {
            phase: "BTB Integration",
            ipc: 0.8,
            hazards: 6,
            branchMispredicts: 8,
            completion: 100
        },
        {
            phase: "RAS Implementation",
            ipc: 1.2,
            hazards: 4,
            branchMispredicts: 6,
            completion: 100
        },
        {
            phase: "FetchBuffer Optimization",
            ipc: 1.5,
            hazards: 2,
            branchMispredicts: 4,
            completion: 100
        },
        {
            phase: "GShare/gSkew (En cours)",
            ipc: 1.8,
            hazards: 1,
            branchMispredicts: 2,
            completion: 30
        }
    ];

    const roadmapStatus = [
        {
            feature: "BTB",
            complete: 100,
            complexity: 80,
            impact: 85
        },
        {
            feature: "RAS",
            complete: 100,
            complexity: 70,
            impact: 75
        },
        {
            feature: "FetchBuffer",
            complete: 100,
            complexity: 65,
            impact: 80
        },
        {
            feature: "GShare/gSkew",
            complete: 30,
            complexity: 90,
            impact: 95
        },
        {
            feature: "Store-Load Hazards",
            complete: 15,
            complexity: 85,
            impact: 90
        }
    ];

    return (
        <div className="space-y-6">
            <Card>
                <CardHeader>
                    <CardTitle>Evolution des Performances Pipeline</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="h-96">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart data={currentMetrics}>
                                <CartesianGrid strokeDasharray="3 3" />
                                <XAxis dataKey="phase" angle={-45} textAnchor="end" height={100} />
                                <YAxis yAxisId="left" />
                                <YAxis yAxisId="right" orientation="right" />
                                <Tooltip />
                                <Legend />
                                <Line yAxisId="left" type="monotone" dataKey="ipc" stroke="#4f46e5" name="IPC" />
                                <Line yAxisId="right" type="monotone" dataKey="hazards" stroke="#ef4444" name="Hazards" />
                                <Line yAxisId="right" type="monotone" dataKey="branchMispredicts" stroke="#f97316" name="Branch Mispredicts" />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>État d'Avancement des Fonctionnalités</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="h-96">
                        <ResponsiveContainer width="100%" height="100%">
                            <BarChart data={roadmapStatus}>
                                <CartesianGrid strokeDasharray="3 3" />
                                <XAxis dataKey="feature" />
                                <YAxis />
                                <Tooltip />
                                <Legend />
                                <Bar dataKey="complete" fill="#4f46e5" name="Complété (%)" />
                                <Bar dataKey="complexity" fill="#ef4444" name="Complexité" />
                                <Bar dataKey="impact" fill="#10b981" name="Impact Attendu" />
                            </BarChart>
                        </ResponsiveContainer>
                    </div>
                </CardContent>
            </Card>

            <div className="mt-6 space-y-4">
                <h3 className="text-lg font-semibold">Observations et Recommandations:</h3>
                <div className="space-y-2 text-gray-600">
                    <p>• <strong>Performances Actuelles:</strong> L'IPC a progressé de 0.5 à 1.5 grâce aux optimisations du fetch stage.</p>
                    <p>• <strong>Points Forts:</strong> Réduction significative des hazards (-75%) et des branch mispredicts (-66%).</p>
                    <p>• <strong>Prochaine Étape Critique:</strong> L'implémentation de GShare/gSkew devrait amener l'IPC au-delà de 1.8.</p>
                    <p>• <strong>Zone d'Attention:</strong> La gestion des Store-Load Hazards nécessite une priorisation accrue.</p>
                </div>
            </div>
        </div>
    );
};

export default PipelineProgress;