@ -0,0 +1,36 @@
using System;
using MuseDotNet.Framework;

namespace test
{
	public class Main : Myth
	{
		protected override void OnBegin()
		{
			base.OnBegin();
		}

		protected override void OnEnd()
		{
			base.OnEnd();
		}

		protected override void OnPlayerAdded(Player newPlayer)
		{
			base.OnPlayerAdded(newPlayer);
		}

		protected override void OnPlayerRemoved(Player leftPlayer)
		{
			base.OnPlayerRemoved(leftPlayer);
		}
	}
}