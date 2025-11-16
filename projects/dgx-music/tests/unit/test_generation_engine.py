"""
Unit Tests for Music Generation Engine
======================================

Tests core generation logic without requiring GPU.
"""

import pytest
from unittest.mock import Mock, patch, MagicMock
import numpy as np
from pathlib import Path

from services.generation.engine import (
    MusicGenerationEngine,
    GenerationError,
    ModelLoadError,
)
from services.generation.models import (
    GenerationRequest,
    GenerationStatus,
    ModelName,
)


class TestMusicGenerationEngine:
    """Test suite for MusicGenerationEngine."""

    @pytest.fixture
    def mock_model(self):
        """Create a mock MusicGen model."""
        model = Mock()
        model.sample_rate = 32000
        model.generate = Mock(return_value=[
            # Mock tensor output (1, channels, samples)
            type('Tensor', (), {
                'cpu': lambda: type('Tensor', (), {
                    'numpy': lambda: np.random.randn(2, 32000 * 8)  # 8s stereo
                })()
            })()
        ])
        return model

    @pytest.fixture
    def engine_no_load(self):
        """Create engine without loading model."""
        with patch('services.generation.engine.MusicGen'):
            engine = MusicGenerationEngine(
                model_name="small",
                use_gpu=False,
                enable_caching=False
            )
            return engine

    def test_engine_initialization(self):
        """Test engine initializes with correct parameters."""
        with patch('services.generation.engine.MusicGen'):
            engine = MusicGenerationEngine(
                model_name="small",
                use_gpu=False,
                enable_caching=False
            )

            assert engine.model_name == "small"
            assert engine.use_gpu is False
            assert engine.enable_caching is False
            assert engine.device == "cpu"
            assert engine.model is None  # Not loaded yet

    def test_cuda_check(self, engine_no_load):
        """Test CUDA availability check."""
        with patch('torch.cuda.is_available', return_value=True):
            assert engine_no_load._check_cuda() is True

        with patch('torch.cuda.is_available', return_value=False):
            assert engine_no_load._check_cuda() is False

    def test_model_loading(self, engine_no_load, mock_model):
        """Test model loads successfully."""
        with patch('services.generation.engine.MusicGen') as MockMusicGen:
            MockMusicGen.get_pretrained = Mock(return_value=mock_model)

            engine_no_load.load_model()

            assert engine_no_load.model is not None
            MockMusicGen.get_pretrained.assert_called_once_with("small")

    def test_model_loading_error(self, engine_no_load):
        """Test model loading handles errors."""
        with patch('services.generation.engine.MusicGen') as MockMusicGen:
            MockMusicGen.get_pretrained = Mock(side_effect=Exception("Load failed"))

            with pytest.raises(ModelLoadError):
                engine_no_load.load_model()

    def test_model_unloading(self, engine_no_load, mock_model):
        """Test model unloads and frees memory."""
        engine_no_load.model = mock_model

        with patch('torch.cuda.is_available', return_value=False):
            engine_no_load.unload_model()

        assert engine_no_load.model is None

    def test_set_generation_params(self, engine_no_load, mock_model):
        """Test generation parameters are set correctly."""
        engine_no_load.model = mock_model

        engine_no_load.set_generation_params(
            duration=16.0,
            temperature=1.5,
            top_k=300,
            top_p=0.9,
            cfg_coef=4.0,
        )

        mock_model.set_generation_params.assert_called_once_with(
            duration=16.0,
            temperature=1.5,
            top_k=300,
            top_p=0.9,
            cfg_coef=4.0,
        )

    def test_generate_audio_success(self, engine_no_load, mock_model):
        """Test successful audio generation."""
        engine_no_load.model = mock_model

        audio, sample_rate = engine_no_load.generate_audio(
            prompt="test prompt",
            duration=8.0,
        )

        assert isinstance(audio, np.ndarray)
        assert audio.shape == (2, 32000 * 8)  # 2 channels, 8 seconds
        assert sample_rate == 32000
        mock_model.generate.assert_called_once_with(["test prompt"])

    def test_generate_audio_loads_model_if_needed(self, engine_no_load, mock_model):
        """Test audio generation loads model if not loaded."""
        with patch.object(engine_no_load, 'load_model') as mock_load:
            engine_no_load.model = None

            with patch.object(engine_no_load, 'set_generation_params'):
                with patch('torch.no_grad'):
                    engine_no_load.model = mock_model  # Set after load_model call
                    mock_load.assert_not_called()  # Not called yet

    def test_generate_audio_error(self, engine_no_load, mock_model):
        """Test audio generation handles errors."""
        mock_model.generate = Mock(side_effect=Exception("Generation failed"))
        engine_no_load.model = mock_model

        with pytest.raises(GenerationError):
            engine_no_load.generate_audio(prompt="test")

    def test_save_audio(self, engine_no_load, tmp_path):
        """Test audio is saved correctly."""
        audio = np.random.randn(2, 32000 * 8)  # 8s stereo
        sample_rate = 32000
        output_path = tmp_path / "test.wav"

        with patch('services.generation.engine.sf.write') as mock_write:
            with patch('services.generation.engine.settings') as mock_settings:
                mock_settings.normalize_audio = False

                metadata = engine_no_load.save_audio(
                    audio=audio,
                    sample_rate=sample_rate,
                    output_path=output_path,
                    normalize=False,
                )

                assert metadata.sample_rate == sample_rate
                assert metadata.channels == 2
                assert metadata.duration == pytest.approx(8.0, rel=0.1)

    def test_normalize_loudness(self, engine_no_load):
        """Test loudness normalization."""
        audio = np.random.randn(32000 * 8, 2)  # 8s stereo (samples, channels)
        sample_rate = 32000

        # Mock pyloudnorm
        with patch('services.generation.engine.pyln') as mock_pyln:
            mock_meter = Mock()
            mock_meter.integrated_loudness = Mock(return_value=-20.0)
            mock_pyln.Meter = Mock(return_value=mock_meter)
            mock_pyln.normalize.loudness = Mock(return_value=audio)

            normalized = engine_no_load._normalize_loudness(
                audio, sample_rate, target_lufs=-16.0
            )

            assert normalized is not None
            mock_pyln.Meter.assert_called_once_with(sample_rate)

    def test_generate_workflow(self, engine_no_load, mock_model, tmp_path):
        """Test complete generation workflow."""
        engine_no_load.model = mock_model

        request = GenerationRequest(
            prompt="test hip hop beat",
            duration=8.0,
            temperature=1.0,
        )

        with patch('services.generation.engine.settings') as mock_settings:
            mock_settings.normalize_audio = False
            mock_settings.get_output_path = Mock(return_value=tmp_path / "test.wav")

            with patch.object(engine_no_load, 'save_audio') as mock_save:
                from services.generation.models import AudioMetadata
                mock_save.return_value = AudioMetadata(
                    duration=8.0,
                    sample_rate=32000,
                    channels=2,
                    file_size_bytes=1024000,
                    file_size_mb=1.0,
                )

                result = engine_no_load.generate(request)

                assert result.status == GenerationStatus.COMPLETED
                assert result.prompt == "test hip hop beat"
                assert result.model == "musicgen-small"
                assert result.metadata is not None
                assert result.generation_time_seconds > 0

    def test_generate_workflow_error(self, engine_no_load):
        """Test generation workflow handles errors."""
        engine_no_load.model = None

        request = GenerationRequest(
            prompt="test prompt",
            duration=8.0,
        )

        with patch.object(engine_no_load, 'load_model', side_effect=Exception("Load failed")):
            result = engine_no_load.generate(request)

            assert result.status == GenerationStatus.FAILED
            assert result.error_message is not None

    def test_engine_stats(self, engine_no_load, mock_model):
        """Test engine statistics tracking."""
        engine_no_load.model = mock_model
        engine_no_load._generation_count = 5
        engine_no_load._total_generation_time = 100.0

        stats = engine_no_load.get_stats()

        assert stats["model_name"] == "musicgen-small"
        assert stats["device"] == "cpu"
        assert stats["model_loaded"] is True
        assert stats["generation_count"] == 5
        assert stats["average_generation_time"] == 20.0


class TestGenerationModels:
    """Test Pydantic models."""

    def test_generation_request_validation(self):
        """Test request validation."""
        # Valid request
        request = GenerationRequest(
            prompt="test prompt",
            duration=16.0,
        )
        assert request.prompt == "test prompt"
        assert request.duration == 16.0

        # Invalid duration (too high)
        with pytest.raises(Exception):
            GenerationRequest(prompt="test", duration=100.0)

        # Invalid prompt (empty)
        with pytest.raises(Exception):
            GenerationRequest(prompt="", duration=16.0)

    def test_generation_request_defaults(self):
        """Test request default values."""
        request = GenerationRequest(prompt="test")

        assert request.duration == 16.0
        assert request.temperature == 1.0
        assert request.top_k == 250
        assert request.top_p == 0.0
        assert request.model == ModelName.MUSICGEN_SMALL


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
