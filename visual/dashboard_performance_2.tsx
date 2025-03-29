import React from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

const PerformanceDashboard = () => {
    const testData = [
        {
            name: "Test 1: Branchements",
            ipc: 0.64,
            execUtil: 127.27,
            memUtil: 18.18,
            stalls: 0,
            hazards: 0,
            forwards: 2,
            description: "Test de branchements"
        },
        {
            name: "Test 2: Hazards",
            ipc: 1.00,
            execUtil: 200.00,
            memUtil: 76.92,
            stalls: 3,
            hazards: 3,
            forwards: 5,
            description: "Test des hazards"
        },
        {
            name: "Test 3: Cache",
            ipc: 1.73,
            execUtil: 345.45,
            memUtil: 181.82,
            stalls: 4,
            hazards: 4,
            forwards: 7,
            description: "Test du cache"
        },
        {
            name: "Test 4: Complexe",
            ipc: 1.81,
            execUtil: 362.50,
            memUtil: 162.50,
            stalls: 6,
            hazards: 6,
            forwards: 11,
            description: "Programme complexe"
        }
    ];

    const hazardData = [
        { name: "Test 2", dataHazards: 1, storeLoadHazards: 2, loadUseHazards: 0 },
        { name: "Test 3", dataHazards: 1, storeLoadHazards: 3, loadUseHazards: 0 },
        { name: "Test 4", dataHazards: 2, storeLoadHazards: 4, loadUseHazards: 0 }
    ];

    return (
        <div className="space-y-6">
            <Card>
                <CardHeader>
                    <CardTitle>Performances du Pipeline</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="h-96">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart data={testData}>
                                <CartesianGrid strokeDasharray="3 3" />
                                <XAxis dataKey="name" angle={-45} textAnchor="end" height={100} />
                                <YAxis yAxisId="left" />
                                <YAxis yAxisId="right" orientation="right" />
                                <Tooltip />
                                <Legend />
                                <Line yAxisId="left" type="monotone" dataKey="ipc" stroke="#4f46e5" name="IPC" />
                                <Line yAxisId="right" type="monotone" dataKey="execUtil" stroke="#10b981" name="Utilisation Exec %" />
                                <Line yAxisId="right" type="monotone" dataKey="memUtil" stroke="#ef4444" name="Utilisation Mém %" />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>

                    <div className="mt-4 grid grid-cols-2 gap-4">
                        <div className="p-4 bg-blue-50 rounded-lg">
                            <h3 className="text-lg font-semibold text-blue-700">Points Forts</h3>
                            <ul className="mt-2 space-y-1 text-sm text-blue-600">
                                <li>IPC maximal de 1.81 atteint</li>
                                <li>Utilisation exec jusqu'à 362.50%</li>
                                <li>Forwarding très efficace (100% réussi)</li>
                                <li>Amélioration continue des performances</li>
                            </ul>
                        </div>
                        <div className="p-4 bg-amber-50 rounded-lg">
                            <h3 className="text-lg font-semibold text-amber-700">Optimisations Possibles</h3>
                            <ul className="mt-2 space-y-1 text-sm text-amber-600">
                                <li>Réduire les store-load hazards</li>
                                <li>Optimiser l'utilisation mémoire</li>
                                <li>Améliorer la prédiction de branchement</li>
                                <li>Réduire les stalls du pipeline</li>
                            </ul>
                        </div>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Analyse des Hazards et Forwarding</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="h-96">
                        <ResponsiveContainer width="100%" height="100%">
                            <BarChart data={testData}>
                                <CartesianGrid strokeDasharray="3 3" />
                                <XAxis dataKey="name" angle={-45} textAnchor="end" height={100} />
                                <YAxis />
                                <Tooltip />
                                <Legend />
                                <Bar dataKey="hazards" fill="#ef4444" name="Hazards" />
                                <Bar dataKey="forwards" fill="#4f46e5" name="Forwarding Réussis" />
                                <Bar dataKey="stalls" fill="#f97316" name="Stalls" />
                            </BarChart>
                        </ResponsiveContainer>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Distribution des Hazards par Type</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="h-96">
                        <ResponsiveContainer width="100%" height="100%">
                            <BarChart data={hazardData}>
                                <CartesianGrid strokeDasharray="3 3" />
                                <XAxis dataKey="name" />
                                <YAxis />
                                <Tooltip />
                                <Legend />
                                <Bar dataKey="dataHazards" fill="#4f46e5" name="Data Hazards" />
                                <Bar dataKey="storeLoadHazards" fill="#ef4444" name="Store-Load Hazards" />
                                <Bar dataKey="loadUseHazards" fill="#f97316" name="Load-Use Hazards" />
                            </BarChart>
                        </ResponsiveContainer>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
};

export default PerformanceDashboard;