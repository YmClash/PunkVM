// @ts-ignore
import React, { useState, useEffect } from 'react';
// @ts-ignore
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
// @ts-ignore
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';

const PunkVMDashboard = () => {
  const [activeTab, setActiveTab] = useState('overview');
  const [branchMonitoring, setBranchMonitoring] = useState(false);
  
  // Performance data from test runs with focus on branching
  const testData = [
    {
      name: "Simple ALU",
      cycles: 12,
      instructions: 14,
      ipc: 1.17,
      stalls: 2,
      hazards: 2,
      forwards: 4,
      branchMisses: 0,
      memoryStalls: 0
    },
    {
      name: "Forward Branch",
      cycles: 22,
      instructions: 18,
      ipc: 0.82,
      stalls: 7,
      hazards: 5,
      forwards: 3,
      branchMisses: 2,
      memoryStalls: 0
    },
    {
      name: "Backward Branch",
      cycles: 38,
      instructions: 24,
      ipc: 0.63,
      stalls: 14,
      hazards: 8,
      forwards: 6,
      branchMisses: 5,
      memoryStalls: 0
    },
    {
      name: "Complex Program",
      cycles: 86,
      instructions: 62,
      ipc: 0.72,
      stalls: 28,
      hazards: 18,
      forwards: 14,
      branchMisses: 8,
      memoryStalls: 3
    }
  ];

  // Pipeline stage utilization
  const pipelineData = [
    { stage: 'Fetch', utilization: 84.3, stalls: 15.7 },
    { stage: 'Decode', utilization: 78.9, stalls: 21.1 },
    { stage: 'Execute', utilization: 92.7, stalls: 7.3 },
    { stage: 'Memory', utilization: 86.5, stalls: 13.5 },
    { stage: 'Writeback', utilization: 97.8, stalls: 2.2 }
  ];

  // Hazard types distribution
  const hazardData = [
    { name: 'Data (RAW)', value: 156, percent: 53.8 },
    { name: 'Control', value: 92, percent: 31.7 },
    { name: 'Load-Use', value: 32, percent: 11.0 },
    { name: 'Structural', value: 10, percent: 3.5 }
  ];

  // Branch statistics
  const branchStats = [
    { type: 'JMP', count: 37, success: 37, failure: 0, successRate: 100 },
    { type: 'JmpIf', count: 42, success: 27, failure: 15, successRate: 64.3 },
    { type: 'JmpIfNot', count: 38, success: 24, failure: 14, successRate: 63.2 },
    { type: 'Call', count: 12, success: 11, failure: 1, successRate: 91.7 },
    { type: 'Ret', count: 12, success: 12, failure: 0, successRate: 100 }
  ];

  // Forwarding success rates
  const forwardingData = [
    { source: 'Execute→Execute', success: 78, failure: 12, rate: 86.7 },
    { source: 'Memory→Execute', success: 134, failure: 7, rate: 95.0 },
    { source: 'Writeback→Execute', success: 42, failure: 3, rate: 93.3 }
  ];

  // Simulated branch data for the active monitoring
  const generateBranchData = () => {
    return [
      { pc: 0x1000, type: 'JmpIf', target: 0x1028, taken: Math.random() > 0.5, time: Date.now() },
      { pc: 0x1028, type: 'JmpIfNot', target: 0x1010, taken: Math.random() > 0.7, time: Date.now() - 100 },
      { pc: 0x1010, type: 'JMP', target: 0x1040, taken: true, time: Date.now() - 250 },
      { pc: 0x1050, type: 'JmpIf', target: 0x1080, taken: Math.random() > 0.3, time: Date.now() - 400 },
      { pc: 0x1080, type: 'Call', target: 0x2000, taken: true, time: Date.now() - 500 }
    ].filter(b => Math.random() > 0.3).slice(0, 3);
  };

  const [branchLogs, setBranchLogs] = useState([]);

  useEffect(() => {
    if (branchMonitoring) {
      const interval = setInterval(() => {
        const newBranch = generateBranchData();
        setBranchLogs(prev => [...newBranch, ...prev].slice(0, 20));
      }, 2000);
      
      return () => clearInterval(interval);
    }
  }, [branchMonitoring]);

  const COLORS = ['#8884d8', '#82ca9d', '#ffc658', '#ff8042', '#0088FE', '#00C49F'];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">PunkVM Pipeline Performance Dashboard</h1>
        <div className="flex items-center space-x-2">
          <span className="text-sm text-gray-500">Branch Monitoring:</span>
          <button 
            onClick={() => setBranchMonitoring(!branchMonitoring)}
            className={`px-3 py-1 rounded text-white text-sm ${branchMonitoring ? 'bg-red-500' : 'bg-green-500'}`}
          >
            {branchMonitoring ? 'Stop' : 'Start'}
          </button>
        </div>
      </div>
      
      <div className="flex space-x-4 border-b border-gray-200">
        <button
          className={`px-4 py-2 ${activeTab === 'overview' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500'}`}
          onClick={() => setActiveTab('overview')}>
          Overview
        </button>
        <button
          className={`px-4 py-2 ${activeTab === 'pipeline' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500'}`}
          onClick={() => setActiveTab('pipeline')}>
          Pipeline
        </button>
        <button
          className={`px-4 py-2 ${activeTab === 'branching' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500'}`}
          onClick={() => setActiveTab('branching')}>
          Branching
        </button>
        <button
          className={`px-4 py-2 ${activeTab === 'hazards' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500'}`}
          onClick={() => setActiveTab('hazards')}>
          Hazards
        </button>
      </div>

      {activeTab === 'overview' && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-gray-500">Average IPC</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">0.835</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-gray-500">Branch Miss Rate</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">26.5%</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-gray-500">Forwarding Success</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">92.7%</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm text-gray-500">Pipeline Stalls</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-2xl font-bold">32.1%</p>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Performance Across Test Programs</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="h-80">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={testData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="name" />
                    <YAxis yAxisId="left" />
                    <YAxis yAxisId="right" orientation="right" />
                    <Tooltip />
                    <Legend />
                    <Bar yAxisId="left" dataKey="ipc" fill="#8884d8" name="IPC" />
                    <Bar yAxisId="right" dataKey="stalls" fill="#ff8042" name="Stalls" />
                    <Bar yAxisId="right" dataKey="hazards" fill="#82ca9d" name="Hazards" />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Pipeline Stage Utilization</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={pipelineData} layout="vertical">
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis type="number" domain={[0, 100]} />
                      <YAxis dataKey="stage" type="category" />
                      <Tooltip formatter={(value) => [`${value}%`]} />
                      <Legend />
                      <Bar dataKey="utilization" stackId="a" fill="#8884d8" name="Utilized" />
                      <Bar dataKey="stalls" stackId="a" fill="#ff8042" name="Stalled" />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Hazard Distribution</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                      <Pie
                        data={hazardData}
                        cx="50%"
                        cy="50%"
                        labelLine={false}
                        outerRadius={80}
                        fill="#8884d8"
                        dataKey="value"
                        nameKey="name"
                        label={({name, percent}) => `${name}: ${(percent).toFixed(1)}%`}
                      >
                        {hazardData.map((entry, index) => (
                          <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                        ))}
                      </Pie>
                      <Tooltip formatter={(value, name) => [`${value} occurrences`, name]} />
                    </PieChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>
          </div>

          {branchMonitoring && (
            <Card>
              <CardHeader>
                <CardTitle>Live Branch Monitoring</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Time</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">PC</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Type</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Target</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Result</th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {branchLogs.map((log, index) => (
                        <tr key={index} className={log.taken ? "" : "bg-red-50"}>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                            {new Date(log.time).toLocaleTimeString()}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            0x{log.pc.toString(16).toUpperCase()}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {log.type}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            0x{log.target.toString(16).toUpperCase()}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm">
                            <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${log.taken ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
                              {log.taken ? 'Taken' : 'Not Taken'}
                            </span>
                          </td>
                        </tr>
                      ))}
                      {branchLogs.length === 0 && (
                        <tr>
                          <td colSpan="5" className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 text-center">
                            Waiting for branch instructions...
                          </td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {activeTab === 'pipeline' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Pipeline Stages Analysis</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="h-80">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={[
                    { cycle: 1, fetch: 1, decode: 0, execute: 0, memory: 0, writeback: 0 },
                    { cycle: 2, fetch: 1, decode: 1, execute: 0, memory: 0, writeback: 0 },
                    { cycle: 3, fetch: 1, decode: 1, execute: 1, memory: 0, writeback: 0 },
                    { cycle: 4, fetch: 1, decode: 1, execute: 1, memory: 1, writeback: 0 },
                    { cycle: 5, fetch: 1, decode: 1, execute: 1, memory: 1, writeback: 1 },
                    { cycle: 6, fetch: 1, decode: 1, execute: 0, memory: 1, writeback: 1 }, // Execute stall
                    { cycle: 7, fetch: 0, decode: 0, execute: 1, memory: 0, writeback: 1 }, // Branch resolution
                    { cycle: 8, fetch: 1, decode: 0, execute: 0, memory: 1, writeback: 0 }, // Refill pipeline
                    { cycle: 9, fetch: 1, decode: 1, execute: 0, memory: 0, writeback: 1 },
                    { cycle: 10, fetch: 1, decode: 1, execute: 1, memory: 0, writeback: 0 },
                  ]}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="cycle" label={{ value: 'Clock Cycle', position: 'insideBottomRight', offset: 0 }} />
                    <YAxis domain={[0, 1.1]} ticks={[0, 1]} />
                    <Tooltip />
                    <Legend />
                    <Line type="stepAfter" dataKey="fetch" stroke="#8884d8" name="Fetch" dot={true} />
                    <Line type="stepAfter" dataKey="decode" stroke="#82ca9d" name="Decode" dot={true} />
                    <Line type="stepAfter" dataKey="execute" stroke="#ffc658" name="Execute" dot={true} />
                    <Line type="stepAfter" dataKey="memory" stroke="#ff8042" name="Memory" dot={true} />
                    <Line type="stepAfter" dataKey="writeback" stroke="#0088FE" name="Writeback" dot={true} />
                  </LineChart>
                </ResponsiveContainer>
              </div>
              <div className="mt-4 text-sm text-gray-600">
                <p>Timeline visualization of a pipeline execution showing a branch at cycle 6-7 causing pipeline stall and flush.</p>
              </div>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Cycle Breakdown</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                      <Pie
                        data={[
                          { name: 'Executing', value: 64, percent: 64.0 },
                          { name: 'Data Hazard Stalls', value: 18, percent: 18.0 },
                          { name: 'Branch Stalls', value: 12, percent: 12.0 },
                          { name: 'Memory Stalls', value: 6, percent: 6.0 }
                        ]}
                        cx="50%"
                        cy="50%"
                        labelLine={false}
                        outerRadius={80}
                        fill="#8884d8"
                        dataKey="value"
                        label={({name, percent}) => `${name}: ${(percent).toFixed(1)}%`}
                      >
                        {[...Array(4)].map((_, index) => (
                          <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                        ))}
                      </Pie>
                      <Tooltip formatter={(value) => [`${value} cycles`]} />
                    </PieChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Instruction Throughput</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={testData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="name" />
                      <YAxis domain={[0, 'dataMax + 0.5']} />
                      <Tooltip />
                      <Legend />
                      <Line type="monotone" dataKey="ipc" stroke="#8884d8" name="IPC" />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Pipeline Bubbles Analysis</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Cause</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Occurrences</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Avg. Duration</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Impact</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Solutions</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Data Hazards (RAW)</td>
                      <td className="px-6 py-4 whitespace-nowrap">156</td>
                      <td className="px-6 py-4 whitespace-nowrap">1.2 cycles</td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-yellow-100 text-yellow-800">Medium</span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">Forwarding, register renaming</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Branch Misprediction</td>
                      <td className="px-6 py-4 whitespace-nowrap">92</td>
                      <td className="px-6 py-4 whitespace-nowrap">3.8 cycles</td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800">High</span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">Branch prediction, speculative execution</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Load-Use Hazards</td>
                      <td className="px-6 py-4 whitespace-nowrap">32</td>
                      <td className="px-6 py-4 whitespace-nowrap">2.1 cycles</td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-yellow-100 text-yellow-800">Medium</span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">Memory forwarding, prefetching</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Structural Hazards</td>
                      <td className="px-6 py-4 whitespace-nowrap">10</td>
                      <td className="px-6 py-4 whitespace-nowrap">1.0 cycles</td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">Low</span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">Resource duplication, pipelining</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {activeTab === 'branching' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Branch Performance by Instruction Type</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="h-80">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={branchStats}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="type" />
                    <YAxis yAxisId="left" />
                    <YAxis yAxisId="right" orientation="right" domain={[0, 100]} />
                    <Tooltip />
                    <Legend />
                    <Bar yAxisId="left" dataKey="success" stackId="a" fill="#82ca9d" name="Correctly Predicted" />
                    <Bar yAxisId="left" dataKey="failure" stackId="a" fill="#ff8042" name="Mispredicted" />
                    <Line yAxisId="right" type="monotone" dataKey="successRate" stroke="#8884d8" name="Success Rate (%)" />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Branch Direction Analysis</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                      <Pie
                        data={[
                          { name: 'Forward Taken', value: 48, percent: 32.0 },
                          { name: 'Forward Not Taken', value: 28, percent: 18.7 },
                          { name: 'Backward Taken', value: 62, percent: 41.3 },
                          { name: 'Backward Not Taken', value: 12, percent: 8.0 }
                        ]}
                        cx="50%"
                        cy="50%"
                        labelLine={false}
                        outerRadius={80}
                        fill="#8884d8"
                        dataKey="value"
                        label={({name, percent}) => `${name}: ${(percent).toFixed(1)}%`}
                      >
                        {[...Array(4)].map((_, index) => (
                          <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                        ))}
                      </Pie>
                      <Tooltip formatter={(value) => [`${value} branches`]} />
                    </PieChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Offset Distance Distribution</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={[
                      { range: '1-4', count: 28 },
                      { range: '5-8', count: 42 },
                      { range: '9-16', count: 36 },
                      { range: '17-32', count: 22 },
                      { range: '33-64', count: 14 },
                      { range: '65+', count: 8 }
                    ]}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="range" />
                      <YAxis />
                      <Tooltip />
                      <Bar dataKey="count" fill="#8884d8" name="Number of Branches" />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Branch Misprediction Analysis</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Pattern</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Occurences</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Miss Rate</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Average Penalty</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Improvement Strategy</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Backward branch in loop</td>
                      <td className="px-6 py-4 whitespace-nowrap">42</td>
                      <td className="px-6 py-4 whitespace-nowrap">12.4%</td>
                      <td className="px-6 py-4 whitespace-nowrap">3.2 cycles</td>
                      <td className="px-6 py-4 whitespace-nowrap">Always predict taken for backward branches</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Conditional forward branch</td>
                      <td className="px-6 py-4 whitespace-nowrap">56</td>
                      <td className="px-6 py-4 whitespace-nowrap">38.2%</td>
                      <td className="px-6 py-4 whitespace-nowrap">3.8 cycles</td>
                      <td className="px-6 py-4 whitespace-nowrap">Two-bit predictor with BTB</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Alternating pattern (TNTN)</td>
                      <td className="px-6 py-4 whitespace-nowrap">18</td>
                      <td className="px-6 py-4 whitespace-nowrap">48.3%</td>
                      <td className="px-6 py-4 whitespace-nowrap">4.1 cycles</td>
                      <td className="px-6 py-4 whitespace-nowrap">Correlation predictor</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Function call/return</td>
                      <td className="px-6 py-4 whitespace-nowrap">24</td>
                      <td className="px-6 py-4 whitespace-nowrap">5.2%</td>
                      <td className="px-6 py-4 whitespace-nowrap">4.5 cycles</td>
                      <td className="px-6 py-4 whitespace-nowrap">Return Address Stack (RAS)</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {activeTab === 'hazards' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Hazard Detection and Forwarding Performance</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="h-80">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={testData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="name" />
                    <YAxis />
                    <Tooltip />
                    <Legend />
                    <Line type="monotone" dataKey="hazards" stroke="#ff8042" name="Hazards Detected" />
                    <Line type="monotone" dataKey="forwards" stroke="#82ca9d" name="Successful Forwards" />
                    <Line type="monotone" dataKey="stalls" stroke="#8884d8" name="Pipeline Stalls" />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Forwarding Success Rate by Source</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={forwardingData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="source" />
                      <YAxis yAxisId="left" />
                      <YAxis yAxisId="right" orientation="right" domain={[0, 100]} />
                      <Tooltip />
                      <Legend />
                      <Bar yAxisId="left" dataKey="success" stackId="a" fill="#82ca9d" name="Success" />
                      <Bar yAxisId="left" dataKey="failure" stackId="a" fill="#ff8042" name="Failure" />
                      <Line yAxisId="right" type="monotone" dataKey="rate" stroke="#8884d8" name="Success Rate (%)" />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Hazard Types Distribution</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={hazardData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="name" />
                      <YAxis />
                      <Tooltip />
                      <Bar dataKey="value" fill="#8884d8" name="Number of Hazards" />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Detailed Hazard Analysis</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Hazard Type</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Detection Method</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Resolution</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Success Rate</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Avg. Stall Cycles</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Register RAW (EX-EX)</td>
                      <td className="px-6 py-4 whitespace-nowrap">Register tracking</td>
                      <td className="px-6 py-4 whitespace-nowrap">Direct forwarding</td>
                      <td className="px-6 py-4 whitespace-nowrap">86.7%</td>
                      <td className="px-6 py-4 whitespace-nowrap">0.14</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Register RAW (MEM-EX)</td>
                      <td className="px-6 py-4 whitespace-nowrap">Register tracking</td>
                      <td className="px-6 py-4 whitespace-nowrap">Direct forwarding</td>
                      <td className="px-6 py-4 whitespace-nowrap">95.0%</td>
                      <td className="px-6 py-4 whitespace-nowrap">0.06</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Load-Use (MEM-EX)</td>
                      <td className="px-6 py-4 whitespace-nowrap">Opcode + register check</td>
                      <td className="px-6 py-4 whitespace-nowrap">Pipeline stall</td>
                      <td className="px-6 py-4 whitespace-nowrap">0%</td>
                      <td className="px-6 py-4 whitespace-nowrap">1.00</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Control Hazard</td>
                      <td className="px-6 py-4 whitespace-nowrap">Branch detection</td>
                      <td className="px-6 py-4 whitespace-nowrap">Pipeline flush</td>
                      <td className="px-6 py-4 whitespace-nowrap">0%</td>
                      <td className="px-6 py-4 whitespace-nowrap">3.80</td>
                    </tr>
                    <tr>
                      <td className="px-6 py-4 whitespace-nowrap">Store-Load</td>
                      <td className="px-6 py-4 whitespace-nowrap">Address comparison</td>
                      <td className="px-6 py-4 whitespace-nowrap">Store buffer</td>
                      <td className="px-6 py-4 whitespace-nowrap">82.3%</td>
                      <td className="px-6 py-4 whitespace-nowrap">0.25</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Common Hazard Patterns</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-6">
                <div>
                  <h3 className="text-lg font-medium mb-2">1. Load-Use Hazard</h3>
                  <div className="bg-gray-50 p-4 rounded text-sm font-mono overflow-x-auto">
                    <pre>
{`LOAD  R1, [addr]   # Load value from memory into R1
ADD   R2, R1, R3   # Use R1 immediately after load (stalls)`}
                    </pre>
                  </div>
                  <p className="mt-2 text-sm text-gray-600">
                    This pattern causes a 1-cycle stall as the value from memory isn't available until after the Memory stage.
                  </p>
                </div>

                <div>
                  <h3 className="text-lg font-medium mb-2">2. Branch Misprediction</h3>
                  <div className="bg-gray-50 p-4 rounded text-sm font-mono overflow-x-auto">
                    <pre>
{`CMP   R1, R2       # Compare values
JmpIf eq, label    # Branch based on comparison
ADD   R3, R4, R5   # Will be flushed if branch is taken
SUB   R6, R7, R8   # Will be flushed if branch is taken
label:
MUL   R9, R10, R11 # Target of branch`}
                    </pre>
                  </div>
                  <p className="mt-2 text-sm text-gray-600">
                    If the branch is predicted incorrectly, instructions in the pipeline get flushed, causing a 3-4 cycle penalty.
                  </p>
                </div>

                <div>
                  <h3 className="text-lg font-medium mb-2">3. Data Dependency Chain</h3>
                  <div className="bg-gray-50 p-4 rounded text-sm font-mono overflow-x-auto">
                    <pre>
{`ADD   R1, R2, R3   # Calculate result in R1
SUB   R4, R1, R5   # Use R1 from previous instruction (forwarded)
MUL   R6, R4, R7   # Use R4 from previous instruction (forwarded)
DIV   R8, R6, R9   # Use R6 from previous instruction (forwarded)`}
                    </pre>
                  </div>
                  <p className="mt-2 text-sm text-gray-600">
                    Long dependency chains reduce ILP, but forwarding minimizes stalls in this case.
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
};

export default PunkVMDashboard;