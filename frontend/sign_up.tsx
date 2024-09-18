function Title() {
    return (
      <h1 className="flex flex-row justify-center p-12 text-[#4caf50] text-6xl font-bold font-fredoka drop-shadow-2xl">
        Ribbit
      </h1>
    );
  }
  
  function Form() {
    return (
      <div className="w-full max-w-md p-6 bg-white rounded-xl shadow">
        <input
          type="text"
          placeholder="Username"
          className="w-full h-12 mb-4 p-3 bg-[#f0f0f0] placeholder-opacity-50 placeholder-gray-500 rounded"
        />
        <input
          type="email"
          placeholder="Email address"
          className="w-full h-12 mb-4 p-3 bg-[#f0f0f0] placeholder-opacity-50 placeholder-gray-500 rounded"
        />
        <input
          type="password"
          placeholder="Password"
          className="w-full h-12 mb-4 p-3 bg-[#f0f0f0] placeholder-opacity-50 placeholder-gray-500 rounded"
        />
        <input
          type="password"
          placeholder="Repeat password"
          className="w-full h-12 mb-4 p-3 bg-[#f0f0f0] placeholder-opacity-50 placeholder-gray-500 rounded"
        />
        <button className="w-full h-12 p-3 bg-[#4caf50] text-white rounded hover:bg-[#45a049]">
          Sign up
        </button>
      </div>
    );
  }
  
  export default function Home() {
    return (
      <div className="flex flex-col items-center  min-h-screen bg-[#F0F0F0]">
        <Title />
        <Form />
      </div>
    );
  }
  