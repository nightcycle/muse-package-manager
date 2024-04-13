
using System.Collections.Generic;
using System;
using MuseDotNet.Framework;
namespace BuildTest
{
	public class StepService
	{
		private static readonly StepService instance = new StepService();
		public Signal<double> OnStep = new();
		public double Time = 0;
		// set up as singleton
		private static StepService instance;
		public static StepService Instance
		{
			get
			{
				instance ??= new StepService();
				return instance;
			}
		}
		private StepService(){}
	}
	
	public class VolumeScriptNameHere : Spawner
	{
		readonly Timer Timer = new();
		double LastUpdate = 0;
		protected override void OnBegin()
		{
			base.OnBegin();
			Timer.Start();
		}
		protected override void OnEnd()
		{
			base.OnEnd();
		}
		public override Actor Spawn()
		{
			StepService.Instance.Time = Timer.ElapsedSeconds;
			double deltaTime = StepService.Instance.Time - LastUpdate;
			if (deltaTime > 0.005){
				LastUpdate = StepService.Instance.Time;
				StepService.Instance.OnStep.Fire(deltaTime);
			}
			Actor actor = base.Spawn();
			actor.Remove();
			return actor;
		}
	}
	// the bound functionality, needs to be disconnected to avoid memory leaks
	public class SignalConnection<V>(Action<V> onInvoke, Action<SignalConnection<V>> onDisconnectInvoke)
	{
		private readonly Action<SignalConnection<V>> OnDisconnectInvoke = onDisconnectInvoke;
		public Action<V> OnInvoke = onInvoke;
		public void Disconnect(){
			this.OnDisconnectInvoke(this);
		}
	}
	// the event to fire
	public class Signal<V> {
		// list of connections the signal iterates through when firing
		private readonly List<SignalConnection<V>> Connections = new();
		private bool IsAlive = true;
		// the internal method passed to all connections to allow them to disconnect
		private void Disconnect(SignalConnection<V> connection){
			if (this.Connections.Contains(connection) == true){
				this.Connections.Remove(connection);
			}
		}
		// get if this signal has anything connected
		public bool HasConnections(){
			return this.Connections.Count > 0;
		}
		// fire all the connections
		public void Fire(V value){
			List<SignalConnection<V>> connections = new(this.Connections);
			foreach (SignalConnection<V> connection in connections){
				connection.OnInvoke.Invoke(value);
			}
		}
		// create a signal connection bound to a specific action
		public SignalConnection<V> Connect(Action<V> onInvoke){
			SignalConnection<V> connection = new SignalConnection<V>(onInvoke, this.Disconnect);
			this.Connections.Add(connection);
			return connection;
		}
		public void Destroy(){
			if (this.IsAlive){
				this.IsAlive = false;
				List<SignalConnection<V>> connections = new(this.Connections);
				foreach (SignalConnection<V> connection in connections){
					connection.Disconnect();
				}
			}
		}
		
		public Signal(){}		
	}
}