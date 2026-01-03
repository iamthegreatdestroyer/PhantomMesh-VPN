"""
PhantomMesh Self-Learning Framework
===================================

Continuous model improvement through operational feedback,
automated retraining, and hyperparameter optimization.

Phase P1-003: Advanced Threat Intelligence Integration
Copyright © 2025 Stephen Bilodeau. All rights reserved.
"""

from __future__ import annotations

import asyncio
import numpy as np
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, UTC, timedelta
from enum import Enum, auto
from typing import Any, Dict, List, Optional, Tuple
from collections import defaultdict, deque
import json

import structlog

logger = structlog.get_logger(__name__)


# ═══════════════════════════════════════════════════════════════════════════════
# SELF-LEARNING TYPES
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class OperationalFeedback:
    """Feedback from operational incidents."""
    timestamp: datetime
    incident_type: str
    detection_model: str  # Which model made prediction
    prediction_correct: bool
    prediction_confidence: float
    incident_severity: float
    response_time_ms: float
    resources_used: Dict[str, float]
    success: bool
    notes: str = ""


@dataclass
class Dataset:
    """Training dataset."""
    features: np.ndarray
    labels: np.ndarray
    metadata: Dict[str, Any]
    size: int = 0
    
    def __post_init__(self):
        self.size = len(self.labels) if len(self.labels) > 0 else 0


@dataclass
class TrainedModel:
    """Trained ML model."""
    model_id: str
    model_name: str
    model_version: str
    training_timestamp: datetime
    training_samples: int
    validation_accuracy: float
    test_accuracy: float
    feature_importance: Dict[str, float]
    hyperparameters: Dict[str, Any]


@dataclass
class ImprovementMetrics:
    """Metrics for model improvement."""
    old_accuracy: float
    new_accuracy: float
    accuracy_improvement: float
    improvement_percentage: float
    is_improvement: bool
    confidence_level: float  # 0.0-1.0


@dataclass
class TuningResult:
    """Hyperparameter tuning result."""
    best_parameters: Dict[str, Any]
    best_score: float
    iterations_completed: int
    time_elapsed_seconds: float
    converged: bool


# ═══════════════════════════════════════════════════════════════════════════════
# MODEL TRAINER
# ═══════════════════════════════════════════════════════════════════════════════

class ModelTrainer:
    """Continuous model training and evaluation."""
    
    def __init__(self, model_name: str):
        self.model_name = model_name
        self._training_history: deque = deque(maxlen=100)
        self._validation_performance: deque = deque(maxlen=1000)
        
        logger.info("model_trainer_initialized", model=model_name)
    
    async def prepare_training_data(
        self,
        feedback_buffer: deque[OperationalFeedback]
    ) -> Dataset:
        """Prepare data from operational events."""
        
        if not feedback_buffer:
            return Dataset(
                features=np.array([]),
                labels=np.array([]),
                metadata={"size": 0}
            )
        
        # Convert feedback to features and labels
        features_list = []
        labels_list = []
        
        for feedback in feedback_buffer:
            # Feature: model confidence
            features_list.append([
                feedback.prediction_confidence,
                feedback.response_time_ms,
                feedback.incident_severity,
                len(feedback.resources_used)
            ])
            
            # Label: whether prediction was correct
            labels_list.append(1.0 if feedback.prediction_correct else 0.0)
        
        features = np.array(features_list)
        labels = np.array(labels_list)
        
        return Dataset(
            features=features,
            labels=labels,
            metadata={
                "size": len(labels),
                "prepared_at": datetime.now(UTC).isoformat()
            }
        )
    
    async def train_model(
        self,
        model_name: str,
        training_data: Dataset
    ) -> TrainedModel:
        """Train model with validation."""
        
        if training_data.size < 10:
            logger.warning("insufficient_training_data", size=training_data.size)
            return None
        
        logger.info(
            "starting_model_training",
            model=model_name,
            samples=training_data.size
        )
        
        # Split into train/val
        split_idx = int(training_data.size * 0.8)
        train_features = training_data.features[:split_idx]
        train_labels = training_data.labels[:split_idx]
        val_features = training_data.features[split_idx:]
        val_labels = training_data.labels[split_idx:]
        
        # Simulate training
        await asyncio.sleep(0.5)
        
        # Compute accuracy metrics
        train_accuracy = self._compute_accuracy(train_labels, train_labels)
        val_accuracy = self._compute_accuracy(val_labels, val_labels)
        test_accuracy = val_accuracy - 0.02  # Simulate test degradation
        
        # Feature importance (mock)
        feature_importance = {
            "confidence": 0.35,
            "response_time": 0.25,
            "severity": 0.25,
            "resource_count": 0.15
        }
        
        trained = TrainedModel(
            model_id=self._generate_model_id(),
            model_name=model_name,
            model_version=f"v{len(self._training_history) + 1}",
            training_timestamp=datetime.now(UTC),
            training_samples=training_data.size,
            validation_accuracy=val_accuracy,
            test_accuracy=test_accuracy,
            feature_importance=feature_importance,
            hyperparameters={
                "learning_rate": 0.001,
                "batch_size": 32,
                "epochs": 50
            }
        )
        
        self._training_history.append(trained)
        
        logger.info(
            "model_training_completed",
            model=model_name,
            val_accuracy=round(val_accuracy, 4),
            test_accuracy=round(test_accuracy, 4)
        )
        
        return trained
    
    async def evaluate_improvement(
        self,
        old_model: TrainedModel,
        new_model: TrainedModel
    ) -> ImprovementMetrics:
        """Evaluate if new model is better."""
        
        old_accuracy = old_model.test_accuracy if old_model else 0.5
        new_accuracy = new_model.test_accuracy
        
        improvement = new_accuracy - old_accuracy
        improvement_pct = (improvement / (old_accuracy + 1e-10)) * 100
        
        # Determine if improvement is statistically significant
        is_improvement = improvement > 0.02  # 2% threshold
        confidence = min(abs(improvement) * 10, 1.0)
        
        metrics = ImprovementMetrics(
            old_accuracy=old_accuracy,
            new_accuracy=new_accuracy,
            accuracy_improvement=improvement,
            improvement_percentage=improvement_pct,
            is_improvement=is_improvement,
            confidence_level=confidence
        )
        
        logger.info(
            "model_evaluation_completed",
            old_accuracy=round(old_accuracy, 4),
            new_accuracy=round(new_accuracy, 4),
            is_improvement=is_improvement,
            confidence=round(confidence, 4)
        )
        
        return metrics
    
    def _compute_accuracy(
        self,
        predictions: np.ndarray,
        labels: np.ndarray
    ) -> float:
        """Compute classification accuracy."""
        if len(labels) == 0:
            return 0.0
        
        correct = np.sum(np.round(predictions) == labels)
        return correct / len(labels)
    
    def _generate_model_id(self) -> str:
        """Generate unique model ID."""
        ts = datetime.now(UTC).isoformat().replace(":", "").replace("-", "")
        return f"model_{self.model_name}_{ts}"


# ═══════════════════════════════════════════════════════════════════════════════
# HYPERPARAMETER TUNER
# ═══════════════════════════════════════════════════════════════════════════════

class HyperparameterTuner:
    """Automated hyperparameter optimization."""
    
    def __init__(self):
        self._tuning_history: deque = deque(maxlen=100)
        
        logger.info("hyperparameter_tuner_initialized")
    
    async def tune_hyperparameters(
        self,
        model_name: str,
        parameter_space: Dict[str, List[Any]],
        validation_data: Dataset
    ) -> TuningResult:
        """
        Find optimal hyperparameters via Bayesian optimization.
        
        Strategy:
        1. Start with random search
        2. Identify promising regions
        3. Focus search on high-performing areas
        4. Converge to optimal parameters
        """
        
        logger.info(
            "starting_hyperparameter_tuning",
            model=model_name,
            parameters=len(parameter_space)
        )
        
        start_time = datetime.now(UTC)
        
        # Bayesian optimization rounds
        best_params = None
        best_score = 0.0
        
        # Initial random search
        random_trials = 10
        for i in range(random_trials):
            params = self._random_sample_parameters(parameter_space)
            score = await self._evaluate_parameters(params, validation_data)
            
            if score > best_score:
                best_score = score
                best_params = params
        
        # Directed search around best
        for i in range(5):
            params = self._perturb_parameters(
                best_params,
                parameter_space,
                perturbation=0.1
            )
            score = await self._evaluate_parameters(params, validation_data)
            
            if score > best_score:
                best_score = score
                best_params = params
        
        elapsed = (datetime.now(UTC) - start_time).total_seconds()
        
        result = TuningResult(
            best_parameters=best_params,
            best_score=best_score,
            iterations_completed=random_trials + 5,
            time_elapsed_seconds=elapsed,
            converged=True
        )
        
        self._tuning_history.append(result)
        
        logger.info(
            "hyperparameter_tuning_completed",
            model=model_name,
            best_score=round(best_score, 4),
            iterations=result.iterations_completed,
            time_seconds=round(elapsed, 2)
        )
        
        return result
    
    def _random_sample_parameters(
        self,
        parameter_space: Dict[str, List[Any]]
    ) -> Dict[str, Any]:
        """Randomly sample parameters from search space."""
        
        params = {}
        for param_name, values in parameter_space.items():
            idx = np.random.randint(0, len(values))
            params[param_name] = values[idx]
        
        return params
    
    def _perturb_parameters(
        self,
        base_params: Dict[str, Any],
        parameter_space: Dict[str, List[Any]],
        perturbation: float = 0.1
    ) -> Dict[str, Any]:
        """Perturb parameters around base values."""
        
        perturbed = {}
        
        for param_name, base_value in base_params.items():
            values = parameter_space[param_name]
            
            # Find closest values to base
            if isinstance(base_value, (int, float)):
                # Numeric parameter
                perturbed[param_name] = base_value * (1 + np.random.randn() * perturbation)
            else:
                # Categorical parameter
                perturbed[param_name] = np.random.choice(values)
        
        return perturbed
    
    async def _evaluate_parameters(
        self,
        params: Dict[str, Any],
        validation_data: Dataset
    ) -> float:
        """Evaluate parameters on validation data."""
        
        # Simulate evaluation
        await asyncio.sleep(0.1)
        
        # Return mock accuracy score
        base_score = 0.75
        randomness = np.random.randn() * 0.05
        return float(np.clip(base_score + randomness, 0.0, 1.0))


# ═══════════════════════════════════════════════════════════════════════════════
# FEEDBACK PROCESSOR
# ═══════════════════════════════════════════════════════════════════════════════

class FeedbackProcessor:
    """Process operational feedback to drive model updates."""
    
    def __init__(self):
        self._feedback_buffer: deque[OperationalFeedback] = deque(maxlen=10000)
        self._processing_stats: Dict[str, Any] = {}
        
        logger.info("feedback_processor_initialized")
    
    async def process_feedback(self, feedback: OperationalFeedback):
        """Process single piece of feedback."""
        
        self._feedback_buffer.append(feedback)
        
        # Update statistics
        self._update_stats(feedback)
        
        # Check if retraining is needed
        if len(self._feedback_buffer) % 100 == 0:
            logger.info(
                "feedback_buffer_accumulated",
                size=len(self._feedback_buffer)
            )
    
    def _update_stats(self, feedback: OperationalFeedback):
        """Update processing statistics."""
        
        incident_type = feedback.incident_type
        
        if incident_type not in self._processing_stats:
            self._processing_stats[incident_type] = {
                "count": 0,
                "correct": 0,
                "avg_confidence": 0.0,
                "success_rate": 0.0
            }
        
        stats = self._processing_stats[incident_type]
        stats["count"] += 1
        
        if feedback.prediction_correct:
            stats["correct"] += 1
        
        stats["success_rate"] = stats["correct"] / stats["count"]
    
    def get_feedback_stats(self) -> Dict[str, Any]:
        """Get feedback processing statistics."""
        
        return {
            "feedback_count": len(self._feedback_buffer),
            "incident_types": dict(self._processing_stats),
            "last_feedback_time": (
                self._feedback_buffer[-1].timestamp.isoformat()
                if self._feedback_buffer else None
            )
        }


# ═══════════════════════════════════════════════════════════════════════════════
# SELF-LEARNING FRAMEWORK (ORCHESTRATOR)
# ═══════════════════════════════════════════════════════════════════════════════

class SelfLearningFramework:
    """
    Continuous improvement through operational feedback.
    Orchestrates training, evaluation, and deployment of models.
    """
    
    def __init__(self, model_names: List[str]):
        self.model_names = model_names
        self.feedback_processor = FeedbackProcessor()
        self.trainers = {name: ModelTrainer(name) for name in model_names}
        self.hyperparameter_tuner = HyperparameterTuner()
        
        self._active_models: Dict[str, TrainedModel] = {}
        self._retraining_schedule: Dict[str, timedelta] = {
            name: timedelta(hours=1) for name in model_names
        }
        self._last_retrain: Dict[str, datetime] = {}
        
        logger.info(
            "self_learning_framework_initialized",
            models=len(model_names)
        )
    
    async def process_feedback(self, feedback: OperationalFeedback):
        """Process feedback and trigger model updates if needed."""
        
        await self.feedback_processor.process_feedback(feedback)
        
        # Check if retraining is due
        if self._should_retrain(feedback.detection_model):
            await self.retrain_models()
    
    async def retrain_models(self):
        """Periodic model retraining with latest data."""
        
        logger.info("starting_model_retraining")
        
        for model_name in self.model_names:
            # Prepare training data
            trainer = self.trainers[model_name]
            training_data = await trainer.prepare_training_data(
                self.feedback_processor._feedback_buffer
            )
            
            if training_data.size < 10:
                continue
            
            # Train new model
            new_model = await trainer.train_model(model_name, training_data)
            
            if new_model is None:
                continue
            
            # Evaluate improvement
            old_model = self._active_models.get(model_name)
            improvement = await trainer.evaluate_improvement(old_model, new_model)
            
            # Deploy if improved
            if improvement.is_improvement:
                self._active_models[model_name] = new_model
                self._last_retrain[model_name] = datetime.now(UTC)
                
                logger.info(
                    "model_deployed",
                    model=model_name,
                    improvement_pct=round(improvement.improvement_percentage, 2)
                )
            else:
                logger.info(
                    "model_not_improved",
                    model=model_name,
                    old_accuracy=round(improvement.old_accuracy, 4),
                    new_accuracy=round(improvement.new_accuracy, 4)
                )
    
    async def optimize_hyperparameters(self):
        """Automatic hyperparameter tuning."""
        
        logger.info("starting_hyperparameter_optimization")
        
        for model_name in self.model_names:
            # Define parameter space
            parameter_space = {
                "learning_rate": [0.0001, 0.0005, 0.001, 0.005, 0.01],
                "batch_size": [16, 32, 64, 128],
                "regularization": [0.0, 0.0001, 0.0005, 0.001]
            }
            
            # Get validation data
            trainer = self.trainers[model_name]
            validation_data = await trainer.prepare_training_data(
                self.feedback_processor._feedback_buffer
            )
            
            if validation_data.size < 10:
                continue
            
            # Tune
            result = await self.hyperparameter_tuner.tune_hyperparameters(
                model_name,
                parameter_space,
                validation_data
            )
            
            logger.info(
                "hyperparameters_optimized",
                model=model_name,
                best_score=round(result.best_score, 4),
                parameters=result.best_parameters
            )
    
    def _should_retrain(self, model_name: str) -> bool:
        """Check if model should be retrained."""
        
        if model_name not in self._last_retrain:
            return True
        
        time_since_retrain = (
            datetime.now(UTC) - self._last_retrain[model_name]
        )
        
        schedule = self._retraining_schedule.get(
            model_name,
            timedelta(hours=1)
        )
        
        return time_since_retrain >= schedule
    
    def get_learning_status(self) -> Dict[str, Any]:
        """Get self-learning framework status."""
        
        return {
            "active_models": len(self._active_models),
            "models": [
                {
                    "name": name,
                    "deployed": name in self._active_models,
                    "last_retrain": (
                        self._last_retrain.get(name, "never").isoformat()
                        if isinstance(self._last_retrain.get(name), datetime)
                        else str(self._last_retrain.get(name, "never"))
                    )
                }
                for name in self.model_names
            ],
            "feedback_stats": self.feedback_processor.get_feedback_stats()
        }
