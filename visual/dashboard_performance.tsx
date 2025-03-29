// @ts-ignore
import React from 'react';
// @ts-ignore
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
// @ts-ignore
import { BarChart, Bar, LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

const PipelineAnalysis = () => {
    const performanceData = [
        {
            testCase: "Test 1: Dépendances",
            ipc: 0.50,
            execUtil: 100.00,
            memUtil: 25.00,
            stalls: 0,
            hazards: 0,
            forwards: 3,
            predictorAccuracy: 95.0,
            description: "Dépendances simples"
        },
        {
            testCase: "Test 2: Hazards mémoire",
            ipc: 0.80,
            execUtil: 160.00,
            memUtil: 60.00,
            stalls: 4,
            hazards: 2,
            forwards: 5,
            predictorAccuracy: 92.5,
            description: "Store-Load hazards"
        },
        {
            testCase: "Test 3: Forwarding intensif",
            ipc: 1.50,
            execUtil: 300.00,
            memUtil: 75.00,
            stalls: 4,
            hazards: 2,
            forwards: 8,
            predictorAccuracy: 97.8,
            description: "Forwarding en chaîne"
        },
        {
            testCase: "Test 4: Programme complexe",
            ipc: 1.21,
            execUtil: 242.11,
            memUtil: 73.68,
            stalls: 12,
            hazards: 6,
            forwards: 14,
            predictorAccuracy: 94.3,
            description: "Mix d'opérations"
        }
    ];

    return (
        <div className="space-y-6">
            <Card>
                <CardHeader>
                    <CardTitle>Performance Globale du Pipeline</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="h-96">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart data={performanceData}>
                                <CartesianGrid strokeDasharray="3 3" />
                                <XAxis dataKey="testCase" angle={-45} textAnchor="end" height={100} />
                                <YAxis yAxisId="left" />
                                <YAxis yAxisId="right" orientation="right" domain={[0, 100]} />
                                <Tooltip />
                                <Legend />
                                <Line yAxisId="left" type="monotone" dataKey="ipc" stroke="#4f46e5" name="IPC" />
                                <Line yAxisId="right" type="monotone" dataKey="predictorAccuracy" stroke="#10b981" name="Précision Prédicteur (%)" />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Efficacité du Pipeline et Gestion des Hazards</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="h-96">
                        <ResponsiveContainer width="100%" height="100%">
                            <BarChart data={performanceData}>
                                <CartesianGrid strokeDasharray="3 3" />
                                <XAxis dataKey="testCase" angle={-45} textAnchor="end" height={100} />
                                <YAxis />
                                <Tooltip />
                                <Legend />
                                <Bar dataKey="forwards" fill="#4f46e5" name="Forwards" />
                                <Bar dataKey="hazards" fill="#ef4444" name="Hazards" />
                                <Bar dataKey="stalls" fill="#f97316" name="Stalls" />
                            </BarChart>
                        </ResponsiveContainer>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Utilisation des Ressources</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="h-96">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart data={performanceData}>
                                <CartesianGrid strokeDasharray="3 3" />
                                <XAxis dataKey="testCase" angle={-45} textAnchor="end" height={100} />
                                <YAxis />
                                <Tooltip />
                                <Legend />
                                <Line type="monotone" dataKey="execUtil" stroke="#4f46e5" name="Utilisation Execute (%)" />
                                <Line type="monotone" dataKey="memUtil" stroke="#ef4444" name="Utilisation Mémoire (%)" />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Analyse des Résultats</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                    <p className="text-lg font-medium">Points clés :</p>
                    <ul className="list-disc pl-6 space-y-2">
                        <li className="text-gray-600">L'IPC augmente significativement de 0.5 à 1.5 grâce à l'efficacité du forwarding</li>
                        <li className="text-gray-600">La précision du prédicteur reste au-dessus de 92% même dans les cas complexes</li>
                        <li className="text-gray-600">L'utilisation de l'unité d'exécution atteint 300% dans le test de forwarding intensif</li>
                        <li className="text-gray-600">Le nombre de forwards augmente progressivement, démontrant l'efficacité du mécanisme</li>
                    </ul>
                </CardContent>
            </Card>
        </div>
    );
};

export default PipelineAnalysis;