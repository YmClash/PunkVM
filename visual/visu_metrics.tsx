import React from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

const PerformanceMetrics = () => {
    const performanceData = [
        { metric: 'IPC moyen', value: 1.0, min: 0.5, max: 1.5 },
        { metric: 'Taux de forwarding', value: 0.9, min: 0.0, max: 1.0 },
        { metric: 'Utilisation pipeline', value: 2.0, min: 1.0, max: 3.0 },
        { metric: 'Réduction stalls', value: 0.85, min: 0.0, max: 1.0 }
    ];

    const data = performanceData.map(item => ({
        name: item.metric,
        actuel: (item.value - item.min) / (item.max - item.min) * 100,
        objectif: 100
    }));

    return (
        <Card className="w-full">
        <CardHeader>
            <CardTitle className="text-xl font-bold">Métriques de Performance PunkVM</CardTitle>
    </CardHeader>
    <CardContent>
    <div className="h-96">
    <ResponsiveContainer width="100%" height="100%">
    <BarChart
        data={data}
    layout="vertical"
    margin={{ top: 20, right: 30, left: 120, bottom: 5 }}
>
    <CartesianGrid strokeDasharray="3 3" />
    <XAxis type="number" domain={[0, 100]} unit="%" />
    <YAxis dataKey="name" type="category" />
    <Tooltip />
    <Legend />
    <Bar dataKey="actuel" fill="#4f46e5" name="Performance Actuelle" />
    <Bar dataKey="objectif" fill="#e5e7eb" name="Objectif" opacity={0.3} />
    </BarChart>
    </ResponsiveContainer>
    </div>
    </CardContent>
    </Card>
);
};

export default PerformanceMetrics;