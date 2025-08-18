import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Alert, AlertDescription } from './ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';

interface PredictionRequest {
  lottery_type: string;
  algorithm: string;
  use_ensemble: boolean;
  ensemble_algorithms?: string[];
  historical_days?: number;
}

interface PredictionResponse {
  predicted_numbers: number[];
  predicted_special_numbers?: number[];
  confidence_scores: number[];
  algorithm_metadata: Record<string, any>;
  computation_time_ms: number;
}

interface TrainingRequest {
  lottery_type: string;
  algorithms: string[];
  historical_days: number;
  validation_split: number;
}

const LotteryPrediction: React.FC = () => {
  const [lotteryType, setLotteryType] = useState<string>('Ssq');
  const [algorithm, setAlgorithm] = useState<string>('statistical');
  const [useEnsemble, setUseEnsemble] = useState<boolean>(false);
  const [historicalDays, setHistoricalDays] = useState<number>(365);
  const [prediction, setPrediction] = useState<PredictionResponse | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [availableAlgorithms, setAvailableAlgorithms] = useState<string[]>([]);
  const [trainingResults, setTrainingResults] = useState<Record<string, number>>({});

  const lotteryTypes = [
    { value: 'Ssq', label: '双色球' },
    { value: 'Dlt', label: '大乐透' },
    { value: 'Fc3d', label: '福彩3D' },
    { value: 'Pl3', label: '排列3' },
    { value: 'Pl5', label: '排列5' },
  ];

  const algorithms = [
    { value: 'statistical', label: '统计分析' },
    { value: 'random_forest', label: '随机森林' },
    { value: 'neural_network', label: '神经网络' },
    { value: 'lstm', label: 'LSTM' },
    { value: 'arima', label: 'ARIMA' },
    { value: 'hybrid', label: '混合集成' },
  ];

  useEffect(() => {
    loadAvailableAlgorithms();
  }, [lotteryType]);

  const loadAvailableAlgorithms = async () => {
    try {
      const response = await invoke('get_available_algorithms', { lotteryType });
      if (response.success) {
        setAvailableAlgorithms(response.data || []);
      }
    } catch (err) {
      console.error('Failed to load algorithms:', err);
    }
  };

  const handlePredict = async () => {
    setLoading(true);
    setError(null);
    setPrediction(null);

    try {
      const request: PredictionRequest = {
        lottery_type: lotteryType,
        algorithm: algorithm,
        use_ensemble: useEnsemble,
        historical_days: historicalDays,
      };

      const response = await invoke('predict_numbers', { request });
      if (response.success) {
        setPrediction(response.data);
      } else {
        setError(response.message);
      }
    } catch (err) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const handleTrain = async () => {
    setLoading(true);
    setError(null);

    try {
      const request: TrainingRequest = {
        lottery_type: lotteryType,
        algorithms: [algorithm],
        historical_days: historicalDays,
        validation_split: 0.2,
      };

      const response = await invoke('train_algorithms', { request });
      if (response.success) {
        setTrainingResults(response.data || {});
      } else {
        setError(response.message);
      }
    } catch (err) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const formatNumbers = (numbers: number[], special?: number[]) => {
    const mainNumbers = numbers.join(', ');
    const specialNumbers = special ? special.join(', ') : '';
    return special ? `${mainNumbers} + ${specialNumbers}` : mainNumbers;
  };

  return (
    <div className="max-w-4xl mx-auto p-6">
      <Card>
        <CardHeader>
          <CardTitle>彩票预测系统</CardTitle>
          <CardDescription>
            基于机器学习算法的智能彩票号码预测
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="predict" className="w-full">
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="predict">号码预测</TabsTrigger>
              <TabsTrigger value="train">模型训练</TabsTrigger>
            </TabsList>
            
            <TabsContent value="predict">
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">彩票类型</label>
                    <select 
                      value={lotteryType} 
                      onChange={(e) => setLotteryType(e.target.value)}
                      className="w-full px-3 py-2 border rounded-md bg-background text-foreground"
                    >
                      {lotteryTypes.map(type => (
                        <option key={type.value} value={type.value}>
                          {type.label}
                        </option>
                      ))}
                    </select>
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium mb-2">算法</label>
                    <select 
                      value={algorithm} 
                      onChange={(e) => setAlgorithm(e.target.value)}
                      className="w-full px-3 py-2 border rounded-md bg-background text-foreground"
                    >
                      {algorithms.map(alg => (
                        <option key={alg.value} value={alg.value}>
                          {alg.label}
                        </option>
                      ))}
                    </select>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">历史天数</label>
                    <input
                      type="number"
                      value={historicalDays}
                      onChange={(e) => setHistoricalDays(Number(e.target.value))}
                      min="30"
                      max="2000"
                      className="w-full px-3 py-2 border rounded-md"
                    />
                  </div>
                  
                  <div className="flex items-end">
                    <label className="flex items-center space-x-2">
                      <input
                        type="checkbox"
                        checked={useEnsemble}
                        onChange={(e) => setUseEnsemble(e.target.checked)}
                        className="rounded"
                      />
                      <span className="text-sm">使用集成算法</span>
                    </label>
                  </div>
                </div>

                <div className="flex space-x-4">
                  <Button 
                    onClick={handlePredict} 
                    disabled={loading}
                    className="flex-1"
                  >
                    {loading ? '预测中...' : '开始预测'}
                  </Button>
                  
                  <Button 
                    onClick={handleTrain} 
                    disabled={loading}
                    variant="outline"
                    className="flex-1"
                  >
                    {loading ? '训练中...' : '训练模型'}
                  </Button>
                </div>

                {error && (
                  <Alert variant="destructive">
                    <AlertDescription>{error}</AlertDescription>
                  </Alert>
                )}

                {prediction && (
                  <Card className="mt-4">
                    <CardHeader>
                      <CardTitle>预测结果</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-4">
                        <div>
                          <h4 className="font-medium mb-2">预测号码：</h4>
                          <div className="text-2xl font-bold text-blue-600">
                            {formatNumbers(
                              prediction.predicted_numbers, 
                              prediction.predicted_special_numbers
                            )}
                          </div>
                        </div>
                        
                        <div className="grid grid-cols-2 gap-4">
                          <div>
                            <h4 className="font-medium">置信度：</h4>
                            <div className="text-sm text-gray-600">
                              {prediction.confidence_scores.map((score, index) => 
                                `号码${index + 1}: ${(score * 100).toFixed(1)}%`
                              ).join(', ')}
                            </div>
                          </div>
                          
                          <div>
                            <h4 className="font-medium">计算时间：</h4>
                            <div className="text-sm text-gray-600">
                              {prediction.computation_time_ms}ms
                            </div>
                          </div>
                        </div>
                        
                        <div>
                          <h4 className="font-medium">算法信息：</h4>
                          <div className="text-sm text-gray-600">
                            {Object.entries(prediction.algorithm_metadata).map(([key, value]) => (
                              <div key={key}>{key}: {String(value)}</div>
                            ))}
                          </div>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                )}

                {Object.keys(trainingResults).length > 0 && (
                  <Card className="mt-4">
                    <CardHeader>
                      <CardTitle>训练结果</CardTitle>
                    </CardHeader>
                    <CardContent>
                      {Object.entries(trainingResults).map(([algo, accuracy]) => (
                        <div key={algo} className="flex justify-between">
                          <span>{algo}</span>
                          <span className="font-medium">{(accuracy * 100).toFixed(1)}%</span>
                        </div>
                      ))}
                    </CardContent>
                  </Card>
                )}
              </div>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  );
};

export default LotteryPrediction;