// import React from 'react';
// import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
// import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, LineChart, Line } from 'recharts';
//
// const PerformanceAnalysis = () => {
//     const testData = [
//         {
//             name: "Test 1: Dépendances",
//             cycles: 8,
//             instructions: 4,
//             ipc: 0.50,
//             stalls: 0,
//             execUtilization: 100.00,
//             memUtilization: 25.00,
//             hazards: 0,
//             forwards: 3
//         },
//         {
//             name: "Test 2: Hazards mémoire",
//             cycles: 10,
//             instructions: 8,
//             ipc: 0.80,
//             stalls: 4,
//             execUtilization: 160.00,
//             memUtilization: 60.00,
//             hazards: 2,
//             forwards: 5
//         },
//         {
//             name: "Test 3: Forwarding intensif",
//             cycles: 8,
//             instructions: 12,
//             ipc: 1.50,
//             stalls: 4,
//             execUtilization: 300.00,
//             memUtilization: 75.00,
//             hazards: 2,
//             forwards: 8
//         },
//         {
//             name: "Test 4: Programme complexe",
//             cycles: 19,
//             instructions: 23,
//             ipc: 1.21,
//             stalls: 12,
//             execUtilization: 242.11,
//             memUtilization: 73.68,
//             hazards: 6,
//             forwards: 14
//         }
//     ];
//
//     return (
//         <div className="space-y-6">
//             <Card>
//                 <CardHeader>
//                     <CardTitle>IPC et Hazards par Test</CardTitle>
//                 </CardHeader>
//                 <CardContent>
//                     <div className="h-96">
//                         <ResponsiveContainer width="100%" height="100%">
//                             <BarChart data={testData}>
//                                 <CartesianGrid strokeDasharray="3 3" />
//                                 <XAxis dataKey="name" angle={-45} textAnchor="end" height={100} />
//                                 <YAxis yAxisId="left" />
//                                 <YAxis yAxisId="right" orientation="right" />
//                                 <Tooltip />
//                                 <Legend />
//                                 <Bar yAxisId="left" dataKey="ipc" fill="#4f46e5" name="IPC" />
//                                 <Bar yAxisId="right" dataKey="hazards" fill="#ef4444" name="Hazards" />
//                             </BarChart>
//                         </ResponsiveContainer>
//                     </div>
//                 </CardContent>
//             </Card>
//
//             <Card>
//                 <CardHeader>
//                     <CardTitle>Utilisation des Étages</CardTitle>
//                 </CardHeader>
//                 <CardContent>
//                     <div className="h-96">
//                         <ResponsiveContainer width="100%" height="100%">
//                             <LineChart data={testData}>
//                                 <CartesianGrid strokeDasharray="3 3" />
//                                 <XAxis dataKey="name" angle={-45} textAnchor="end" height={100} />
//                                 <YAxis />
//                                 <Tooltip />
//                                 <Legend />
//                                 <Line type="monotone" dataKey="execUtilization" stroke="#4f46e5" name="Execute %" />
//                                 <Line type="monotone" dataKey="memUtilization" stroke="#ef4444" name="Memory %" />
//                             </LineChart>
//                         </ResponsiveContainer>
//                     </div>
//                 </CardContent>
//             </Card>
//
//             <Card>
//                 <CardHeader>
//                     <CardTitle>Forwarding et Stalls</CardTitle>
//                 </CardHeader>
//                 <CardContent>
//                     <div className="h-96">
//                         <ResponsiveContainer width="100%" height="100%">
//                             <BarChart data={testData}>
//                                 <CartesianGrid strokeDasharray="3 3" />
//                                 <XAxis dataKey="name" angle={-45} textAnchor="end" height={100} />
//                                 <YAxis />
//                                 <Tooltip />
//                                 <Legend />
//                                 <Bar dataKey="forwards" fill="#4f46e5" name="Forwards Réussis" />
//                                 <Bar dataKey="stalls" fill="#ef4444" name="Stalls" />
//                             </BarChart>
//                         </ResponsiveContainer>
//                     </div>
//                 </CardContent>
//             </Card>
//         </div>
//     );
// };
//
// export default PerformanceAnalysis