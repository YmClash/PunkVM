import React, { useState, useEffect } from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, PieChart, Pie, Cell, AreaChart, Area } from 'recharts';
import { CheckCircle, AlertCircle, Clock, Target, Cpu, HardDrive, GitBranch, Bug, Zap } from 'lucide-react';

const PunkVMDashboard = () => {
  const [activeTab, setActiveTab] = useState('status');
  const [liveMetrics, setLiveMetrics] = useState({
    ipc: 0.89,
    cycles: 9,
    instructions: 8,
    stalls: 1,
    hazards: 1,
    cacheHitRate: 98.38,
    branchPredictionRate: 0.00
  });

  // Performance progression data
  const progressionData = [
    { phase: "Pipeline Initial", ipc: 0.50, hazards: 8, completion: 100 },
    { phase: "ALU Implementation", ipc: 0.65, hazards: 6, completion: 100 },
    { phase: "Memory System", ipc: 0.75, hazards: 4, completion: 95 },
    { phase: "Hazard Detection", ipc: 0.82, hazards: 3, completion: 90 },
    { phase: "Forwarding Unit", ipc: 0.89, hazards: 1, completion: 85 },
    { phase: "Branch Prediction", ipc: 0.89, hazards: 1, completion: 70 },
    { phase: "Current Status", ipc: 0.89, hazards: 1, completion: 75 }
  ];

  // Issue tracking data
  const currentIssues = [
    {
      id: 1,
      title: "Calcul d'Adresse de Saut Incorrect",
      type: "bug",
      priority: "critical",
      status: "investigating",
      description: "L'adresse cible 0x36 au lieu de 0x38 pour les sauts",
      impact: "Empêche l'exécution correcte des programmes avec branchements"
    },
    {
      id: 2,
      title: "Instruction RET Non Implémentée",
      type: "feature",
      priority: "high",
      status: "todo",
      description: "L'instruction de retour de fonction n'est pas gérée",
      impact: "Limitation des appels de fonction"
    },
    {
      id: 3,
      title: "Taux Prédiction Branchement à 0%",
      type: "optimization",
      priority: "medium",
      status: "analyzing",
      description: "Le prédicteur ne fonctionne pas correctement",
      impact: "Performance réduite sur les programmes avec boucles"
    },
    {
      id: 4,
      title: "Cache L1 Non Optimisé",
      type: "optimization",
      priority: "low",
      status: "planned",
      description: "Politique de remplacement LRU basique",
      impact: "Performance mémoire sous-optimale"
    }
  ];

  // Branch instruction test results
  const branchTestResults = [
    { instruction: "JMP", implemented: true, tested: true, working: true },
    { instruction: "JmpIf", implemented: true, tested: true, working: true },
    { instruction: "JmpIfNot", implemented: true, tested: true, working: false },
    { instruction: "JmpIfEqual", implemented: true, tested: true, working: true },
    { instruction: "JmpIfNotEqual", implemented: true, tested: true, working: true },
    { instruction: "JmpIfGreater", implemented: true, tested: true, working: true },
    { instruction: "JmpIfLess", implemented: true, tested: true, working: true },
    { instruction: "JmpIfZero", implemented: true, tested: true, working: true },
    { instruction: "JmpIfNotZero", implemented: true, tested: true, working: true },
    { instruction: "Call", implemented: true, tested: false, working: false },
    { instruction: "Ret", implemented: false, tested: false, working: false }
  ];

  // Component implementation status
  const componentStatus = [
    { name: "Pipeline Base", progress: 100, status: "complete", issues: 0 },
    { name: "ALU", progress: 100, status: "complete", issues: 0 },
    { name: "Fetch Stage", progress: 95, status: "stable", issues: 0 },
    { name: "Decode Stage", progress: 90, status: "stable", issues: 1 },
    { name: "Execute Stage", progress: 85, status: "active", issues: 1 },
    { name: "Memory Stage", progress: 90, status: "stable", issues: 0 },
    { name: "Writeback Stage", progress: 100, status: "complete", issues: 0 },
    { name: "Hazard Detection", progress: 80, status: "active", issues: 0 },
    { name: "Forwarding Unit", progress: 75, status: "active", issues: 0 },
    { name: "Branch Predictor", progress: 60, status: "development", issues: 2 },
    { name: "Cache L1", progress: 85, status: "stable", issues: 1 },
    { name: "Store Buffer", progress: 70, status: "development", issues: 0 }
  ];

  // Test program execution timeline
  const executionTimeline = [
    { cycle: 1, fetch: 1, decode: 0, execute: 0, memory: 0, writeback: 0, event: "Start" },
    { cycle: 2, fetch: 1, decode: 1, execute: 0, memory: 0, writeback: 0, event: "Fill" },
    { cycle: 3, fetch: 1, decode: 1, execute: 1, memory: 0, writeback: 0, event: "Fill" },
    { cycle: 4, fetch: 1, decode: 1, execute: 1, memory: 1, writeback: 0, event: "Fill" },
    { cycle: 5, fetch: 1, decode: 1, execute: 1, memory: 1, writeback: 1, event: "Full Pipeline" },
    { cycle: 6, fetch: 0, decode: 0, execute: 1, memory: 1, writeback: 1, event: "Branch Stall" },
    { cycle: 7, fetch: 1, decode: 0, execute: 0, memory: 1, writeback: 1, event: "Refill" },
    { cycle: 8, fetch: 1, decode: 1, execute: 0, memory: 0, writeback: 1, event: "Recovery" },
    { cycle: 9, fetch: 1, decode: 1, execute: 1, memory: 0, writeback: 0, event: "Normal" }
  ];

  // Performance metrics over time
  const performanceHistory = [
    { session: "Session 1", ipc: 0.50, bugs: 8, features: 12 },
    { session: "Session 2", ipc: 0.65, bugs: 6, features: 18 },
    { session: "Session 3", ipc: 0.75, bugs: 4, features: 24 },
    { session: "Session 4", ipc: 0.89, bugs: 3, features: 28 },
    { session: "Current", ipc: 0.89, bugs: 2, features: 30 }
  ];

  const COLORS = ['#3b82f6', '#ef4444', '#10b981', '#f59e0b', '#8b5cf6', '#ec4899'];

  const getStatusColor = (status) => {
    switch (status) {
      case 'complete': return 'text-green-600 bg-green-100';
      case 'stable': return 'text-blue-600 bg-blue-100';
      case 'active': return 'text-yellow-600 bg-yellow-100';
      case 'development': return 'text-orange-600 bg-orange-100';
      case 'critical': return 'text-red-600 bg-red-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  const getPriorityIcon = (priority) => {
    switch (priority) {
      case 'critical': return <AlertCircle className="w-4 h-4 text-red-500" />;
      case 'high': return <Clock className="w-4 h-4 text-orange-500" />;
      case 'medium': return <Target className="w-4 h-4 text-yellow-500" />;
      case 'low': return <CheckCircle className="w-4 h-4 text-green-500" />;
      default: return <CheckCircle className="w-4 h-4 text-gray-500" />;
    }
  };

  return (
    <div className="space-y-6 p-6 bg-gray-50">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">PunkVM - Tableau de Bord de Développement</h1>
          <p className="text-gray-600 mt-2">Pipeline RISC à 5 étages - État d'Avancement et Métriques</p>
        </div>
        <div className="flex items-center space-x-4">
          <div className="text-right">
            <div className="text-sm text-gray-500">IPC Actuel</div>
            <div className="text-2xl font-bold text-blue-600">{liveMetrics.ipc}</div>
          </div>
          <div className="text-right">
            <div className="text-sm text-gray-500">Taux Cache</div>
            <div className="text-2xl font-bold text-green-600">{liveMetrics.cacheHitRate}%</div>
          </div>
        </div>
      </div>

      <div className="flex space-x-4 border-b border-gray-200">
        <button
          className={`px-4 py-2 font-medium ${activeTab === 'status' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500'}`}
          onClick={() => setActiveTab('status')}>
          <div className="flex items-center space-x-2">
            <CheckCircle className="w-4 h-4" />
            <span>État Global</span>
          </div>
        </button>
        <button
          className={`px-4 py-2 font-medium ${activeTab === 'pipeline' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500'}`}
          onClick={() => setActiveTab('pipeline')}>
          <div className="flex items-center space-x-2">
            <Cpu className="w-4 h-4" />
            <span>Pipeline</span>
          </div>
        </button>
        <button
          className={`px-4 py-2 font-medium ${activeTab === 'branching' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500'}`}
          onClick={() => setActiveTab('branching')}>
          <div className="flex items-center space-x-2">
            <GitBranch className="w-4 h-4" />
            <span>Branchements</span>
          </div>
        </button>
        <button
          className={`px-4 py-2 font-medium ${activeTab === 'issues' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500'}`}
          onClick={() => setActiveTab('issues')}>
          <div className="flex items-center space-x-2">
            <Bug className="w-4 h-4" />
            <span>Issues</span>
          </div>
        </button>
        <button
          className={`px-4 py-2 font-medium ${activeTab === 'performance' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500'}`}
          onClick={() => setActiveTab('performance')}>
          <div className="flex items-center space-x-2">
            <Zap className="w-4 h-4" />
            <span>Performance</span>
          </div>
        </button>
      </div>

      {activeTab === 'status' && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-gray-500">Composants Complets</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-green-600">4/12</div>
                <div className="text-sm text-gray-500">33% du système</div>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-gray-500">Issues Critiques</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-red-600">1</div>
                <div className="text-sm text-gray-500">Adressage branchement</div>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-gray-500">Tests Réussis</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-blue-600">8/11</div>
                <div className="text-sm text-gray-500">Instructions branch</div>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-gray-500">Progression</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-purple-600">75%</div>
                <div className="text-sm text-gray-500">Phase 2 complète</div>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>État des Composants</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="h-96">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={componentStatus} layout="vertical">
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis type="number" domain={[0, 100]} />
                    <YAxis dataKey="name" type="category" width={120} />
                    <Tooltip formatter={(value) => [`${value}%`, 'Progression']} />
                    <Bar dataKey="progress" fill="#3b82f6" name="Progression %" />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Progression Historique</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={progressionData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="phase" angle={-45} textAnchor="end" height={80} />
                      <YAxis yAxisId="left" />
                      <YAxis yAxisId="right" orientation="right" />
                      <Tooltip />
                      <Legend />
                      <Line yAxisId="left" type="monotone" dataKey="ipc" stroke="#3b82f6" name="IPC" />
                      <Line yAxisId="right" type="monotone" dataKey="hazards" stroke="#ef4444" name="Hazards" />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Distribution des Statuts</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                      <Pie
                        data={[
                          { name: 'Complete', value: 4, color: '#10b981' },
                          { name: 'Stable', value: 4, color: '#3b82f6' },
                          { name: 'Active', value: 3, color: '#f59e0b' },
                          { name: 'Development', value: 1, color: '#ef4444' }
                        ]}
                        cx="50%"
                        cy="50%"
                        labelLine={false}
                        outerRadius={80}
                        fill="#8884d8"
                        dataKey="value"
                        label={({name, value}) => `${name}: ${value}`}
                      >
                        {[0,1,2,3].map((index) => (
                          <Cell key={`cell-${index}`} fill={COLORS[index]} />
                        ))}
                      </Pie>
                      <Tooltip />
                    </PieChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}

      {activeTab === 'pipeline' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Timeline d'Exécution du Pipeline</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="h-80">
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={executionTimeline}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="cycle" />
                    <YAxis domain={[0, 1.1]} />
                    <Tooltip 
                      labelFormatter={(value) => `Cycle ${value}`}
                      formatter={(value, name) => [value === 1 ? 'Actif' : 'Inactif', name]}
                    />
                    <Legend />
                    <Area type="stepAfter" dataKey="fetch" stackId="1" stroke="#3b82f6" fill="#3b82f6" fillOpacity={0.8} name="Fetch" />
                    <Area type="stepAfter" dataKey="decode" stackId="1" stroke="#10b981" fill="#10b981" fillOpacity={0.8} name="Decode" />
                    <Area type="stepAfter" dataKey="execute" stackId="1" stroke="#f59e0b" fill="#f59e0b" fillOpacity={0.8} name="Execute" />
                    <Area type="stepAfter" dataKey="memory" stackId="1" stroke="#ef4444" fill="#ef4444" fillOpacity={0.8} name="Memory" />
                    <Area type="stepAfter" dataKey="writeback" stackId="1" stroke="#8b5cf6" fill="#8b5cf6" fillOpacity={0.8} name="Writeback" />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
              <div className="mt-4 grid grid-cols-5 gap-2 text-sm">
                {executionTimeline.map(event => (
                  <div key={event.cycle} className="text-center">
                    <div className="font-medium">C{event.cycle}</div>
                    <div className="text-gray-500">{event.event}</div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2">
                  <Cpu className="w-5 h-5" />
                  <span>Métriques Actuelles</span>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span>Cycles</span>
                    <span className="font-bold">{liveMetrics.cycles}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Instructions</span>
                    <span className="font-bold">{liveMetrics.instructions}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>IPC</span>
                    <span className="font-bold text-blue-600">{liveMetrics.ipc}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Stalls</span>
                    <span className="font-bold text-red-600">{liveMetrics.stalls} (11.11%)</span>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2">
                  <HardDrive className="w-5 h-5" />
                  <span>Système Mémoire</span>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span>Cache L1</span>
                    <span className="font-bold text-green-600">{liveMetrics.cacheHitRate}%</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Store Buffer</span>
                    <span className="font-bold">8 entrées</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Mémoire</span>
                    <span className="font-bold">1MB</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Registres</span>
                    <span className="font-bold">16 × 64 bits</span>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2">
                  <GitBranch className="w-5 h-5" />
                  <span>Contrôle de Flux</span>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span>Prédicteur</span>
                    <span className="font-bold">2-bit dynamique</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Précision</span>
                    <span className="font-bold text-red-600">{liveMetrics.branchPredictionRate}%</span>
                  </div>
                  <div className="flex justify-between">
                    <span>BTB</span>
                    <span className="font-bold">64 entrées</span>
                  </div>
                  <div className="flex justify-between">
                    <span>RAS</span>
                    <span className="font-bold">8 niveaux</span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}

      {activeTab === 'branching' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>État des Instructions de Branchement</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Instruction</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Implémenté</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Testé</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Fonctionnel</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Notes</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {branchTestResults.map((result, index) => (
                      <tr key={index}>
                        <td className="px-6 py-4 whitespace-nowrap font-mono">{result.instruction}</td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${result.implemented ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
                            {result.implemented ? 'Oui' : 'Non'}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${result.tested ? 'bg-blue-100 text-blue-800' : 'bg-gray-100 text-gray-800'}`}>
                            {result.tested ? 'Oui' : 'Non'}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${result.working ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
                            {result.working ? 'Oui' : 'Non'}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {result.instruction === 'JmpIfNot' && !result.working ? 'Problème d\'adressage' : ''}
                          {result.instruction === 'Call' && !result.working ? 'Non testé' : ''}
                          {result.instruction === 'Ret' && !result.implemented ? 'À implémenter' : ''}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Résultats des Tests</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                      <Pie
                        data={[
                          { name: 'Fonctionnelles', value: 8, color: '#10b981' },
                          { name: 'Non Fonctionnelles', value: 3, color: '#ef4444' }
                        ]}
                        cx="50%"
                        cy="50%"
                        labelLine={false}
                        outerRadius={80}
                        fill="#8884d8"
                        dataKey="value"
                        label={({name, value}) => `${name}: ${value}`}
                      >
                        <Cell fill="#10b981" />
                        <Cell fill="#ef4444" />
                      </Pie>
                      <Tooltip />
                    </PieChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Programme de Test punk_program_3</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2 text-sm">
                  <div className="bg-gray-50 p-3 rounded">
                    <div className="font-medium">✅ Section 1: Initialisation</div>
                    <div className="text-gray-600">MOV R0-R7 avec valeurs de test</div>
                  </div>
                  <div className="bg-gray-50 p-3 rounded">
                    <div className="font-medium">✅ Section 2: JMP Inconditionnel</div>
                    <div className="text-gray-600">Test de saut simple</div>
                  </div>
                  <div className="bg-red-50 p-3 rounded">
                    <div className="font-medium">❌ Section 3-10: Branchements Conditionnels</div>
                    <div className="text-red-600">Erreur d'adressage 0x36 vs 0x38</div>
                  </div>
                  <div className="bg-gray-50 p-3 rounded">
                    <div className="font-medium">⏳ Section 11-13: Boucles et Fonctions</div>
                    <div className="text-gray-600">Non testé à cause de l'erreur précédente</div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}

      {activeTab === 'issues' && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <Card className="border-red-200">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-red-600">Critiques</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-red-600">1</div>
              </CardContent>
            </Card>
            <Card className="border-orange-200">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-orange-600">Élevées</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-orange-600">1</div>
              </CardContent>
            </Card>
            <Card className="border-yellow-200">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-yellow-600">Moyennes</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-yellow-600">1</div>
              </CardContent>
            </Card>
            <Card className="border-green-200">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-green-600">Faibles</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-green-600">1</div>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Issues Actuelles</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {currentIssues.map((issue) => (
                  <div key={issue.id} className="border rounded-lg p-4">
                    <div className="flex items-start justify-between">
                      <div className="flex items-start space-x-3">
                        {getPriorityIcon(issue.priority)}
                        <div>
                          <h3 className="font-medium text-gray-900">{issue.title}</h3>
                          <p className="text-sm text-gray-600 mt-1">{issue.description}</p>
                          <p className="text-xs text-red-600 mt-2">Impact: {issue.impact}</p>
                        </div>
                      </div>
                      <div className="flex flex-col items-end space-y-2">
                        <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(issue.status)}`}>
                          {issue.status}
                        </span>
                        <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(issue.priority)}`}>
                          {issue.priority}
                        </span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Analyse de l'Issue Critique: Calcul d'Adresse</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                  <h4 className="font-medium text-red-900">Problème Identifié</h4>
                  <div className="mt-2 text-sm text-red-700">
                    <p>• Instruction JMP à l'adresse 0x2A (taille 8 bytes)</p>
                    <p>• Devrait sauter à 0x38, mais tente d'accéder à 0x36</p>
                    <p>• Erreur dans le calcul : current_address + 6 + 6 au lieu de current_address + 8 + 6</p>
                  </div>
                </div>
                
                <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                  <h4 className="font-medium text-blue-900">Solution Proposée</h4>
                  <div className="mt-2 text-sm text-blue-700">
                    <p>1. Corriger la fonction calculate_branch_offset() dans le générateur de test</p>
                    <p>2. Utiliser instruction.total_size() au lieu de valeurs hardcodées</p>
                    <p>3. Ajouter des logs de validation des adresses calculées</p>
                    <p>4. Créer des tests unitaires pour le calcul d'offset</p>
                  </div>
                </div>

                <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                  <h4 className="font-medium text-gray-900">Code Proposé</h4>
                  <pre className="mt-2 text-sm text-gray-700 overflow-x-auto">
{`fn calculate_branch_offset(current_addr: u32, target_addr: u32, instr_size: u32) -> i32 {
    let next_pc = current_addr + instr_size;
    (target_addr as i32) - (next_pc as i32)
}`}
                  </pre>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {activeTab === 'performance' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Évolution des Performances</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="h-80">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={performanceHistory}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="session" />
                    <YAxis yAxisId="left" />
                    <YAxis yAxisId="right" orientation="right" />
                    <Tooltip />
                    <Legend />
                    <Line yAxisId="left" type="monotone" dataKey="ipc" stroke="#3b82f6" name="IPC" strokeWidth={2} />
                    <Line yAxisId="right" type="monotone" dataKey="bugs" stroke="#ef4444" name="Bugs" strokeWidth={2} />
                    <Line yAxisId="right" type="monotone" dataKey="features" stroke="#10b981" name="Features" strokeWidth={2} />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Benchmarks de Performance</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <span>IPC Cible</span>
                    <span className="font-bold">1.2</span>
                  </div>
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div className="bg-blue-600 h-2 rounded-full" style={{width: '74.2%'}}></div>
                  </div>
                  <div className="text-sm text-gray-500">Actuel: 0.89 (74.2%)</div>
                </div>
                
                <div className="space-y-3 mt-4">
                  <div className="flex justify-between items-center">
                    <span>Cache Hit Rate</span>
                    <span className="font-bold">99%</span>
                  </div>
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div className="bg-green-600 h-2 rounded-full" style={{width: '99.4%'}}></div>
                  </div>
                  <div className="text-sm text-gray-500">Actuel: 98.38% (99.4%)</div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Comparaison Architectures</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3 text-sm">
                  <div className="flex justify-between">
                    <span>PunkVM (Actuel)</span>
                    <span className="font-bold">0.89 IPC</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Pipeline Idéal</span>
                    <span className="text-gray-500">1.0 IPC</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Superscalaire 2-way</span>
                    <span className="text-gray-500">1.8 IPC</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Out-of-order</span>
                    <span className="text-gray-500">2.5 IPC</span>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Optimisations Futures</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2 text-sm">
                  <div className="flex items-center justify-between">
                    <span>Branch Prediction</span>
                    <span className="text-green-600">+15% IPC</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span>Out-of-order</span>
                    <span className="text-blue-600">+40% IPC</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span>Superscalaire</span>
                    <span className="text-purple-600">+60% IPC</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span>JIT Compilation</span>
                    <span className="text-orange-600">+200% IPC</span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Analyse Détaillée des Stalls</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Type de Stall</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Occurrences</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Cycles Perdus</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">% du Total</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Solution</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Branch Misprediction</td>
                      <td className="px-6 py-4 whitespace-nowrap">0</td>
                      <td className="px-6 py-4 whitespace-nowrap">0</td>
                      <td className="px-6 py-4 whitespace-nowrap">0%</td>
                      <td className="px-6 py-4 whitespace-nowrap">Corriger le prédicteur</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Data Hazards</td>
                      <td className="px-6 py-4 whitespace-nowrap">1</td>
                      <td className="px-6 py-4 whitespace-nowrap">1</td>
                      <td className="px-6 py-4 whitespace-nowrap">11.11%</td>
                      <td className="px-6 py-4 whitespace-nowrap">Forwarding amélioré</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Load-Use</td>
                      <td className="px-6 py-4 whitespace-nowrap">0</td>
                      <td className="px-6 py-4 whitespace-nowrap">0</td>
                      <td className="px-6 py-4 whitespace-nowrap">0%</td>
                      <td className="px-6 py-4 whitespace-nowrap">Store buffer</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Structural</td>
                      <td className="px-6 py-4 whitespace-nowrap">0</td>
                      <td className="px-6 py-4 whitespace-nowrap">0</td>
                      <td className="px-6 py-4 whitespace-nowrap">0%</td>
                      <td className="px-6 py-4 whitespace-nowrap">Pipeline dupliqué</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      <div className="mt-8 text-center text-gray-500 text-sm">
        <p>PunkVM Development Dashboard | Last updated: {new Date().toLocaleDateString('fr-FR')}</p>
        <p>Pipeline RISC 5-stages | Session de débogage branchements</p>
      </div>
    </div>
  );
};

export default PunkVMDashboard;