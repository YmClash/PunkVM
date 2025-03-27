import React from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

const PerformanceAnalysis = () => {
    const performanceData = [
        {
            testCase: "Test 1: Dépendances",
            ipc: 0.50,
            execUtil: 100.00,
            memUtil: 25.00,
            stalls: 0,
            hazards: 0,
            forwards: 3,
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
            description: "Mix d'opérations"
        }
    ];

    return (
        <div className="space-y-6">
            <Card>
                <CardHeader>
                    <CardTitle>Efficacité Pipeline vs Utilisation Ressources</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="h-96">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart data={performanceData}>
                                <CartesianGrid strokeDasharray="3 3" />
                                <XAxis dataKey="testCase" angle={-45} textAnchor="end" height={100} />
                                <YAxis yAxisId="left" />
                                <YAxis yAxisId="right" orientation="right" />
                                <Tooltip />
                                <Legend />
                                <Line yAxisId="left" type="monotone" dataKey="ipc" stroke="#4f46e5" name="IPC" />
                                <Line yAxisId="right" type="monotone" dataKey="execUtil" stroke="#10b981" name="Exec %" />
                                <Line yAxisId="right" type="monotone" dataKey="memUtil" stroke="#ef4444" name="Mem %" />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Hazards et Forwarding</CardTitle>
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
                                <Bar dataKey="hazards" fill="#ef4444" name="Hazards" />
                                <Bar dataKey="forwards" fill="#4f46e5" name="Forwards" />
                                <Bar dataKey="stalls" fill="#f97316" name="Stalls" />
                            </BarChart>
                        </ResponsiveContainer>
                    </div>

                    <div className="mt-6 space-y-4">
                        <h3 className="text-lg font-semibold">Observations Clés :</h3>

                        <div className="space-y-2">
                            <p className="font-medium">1. Évolution de l'IPC</p>
                            <p className="text-gray-600">
                                - Augmentation progressive : 0.50 → 0.80 → 1.50 → 1.21
                                - Pic à 1.50 avec forwarding intensif
                                - Légère baisse à 1.21 sur le programme complexe due aux hazards
                            </p>

                            <p className="font-medium">2. Efficacité du Forwarding</p>
                            <p className="text-gray-600">
                                - Augmentation constante des forwards : 3 → 5 → 8 → 14
                                - Forte corrélation avec l'utilisation du pipeline
                                - Réduction efficace des stalls
                            </p>

                            <p className="font-medium">3. Impact des Hazards</p>
                            <p className="text-gray-600">
                                - Test 1 : Aucun hazard grâce au forwarding
                                - Tests 2-3 : 2 hazards chacun
                                - Test 4 : 6 hazards mais IPC maintenu grâce au forwarding
                            </p>

                            <p className="font-medium">4. Utilisation des Ressources</p>
                            <p className="text-gray-600">
                                - Execute : Pic à 300% dans le test 3
                                - Memory : Augmentation progressive jusqu'à ~75%
                                - Bonne balance entre execute et memory dans le test 4
                            </p>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
};

export default PerformanceAnalysis;